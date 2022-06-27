import * as path from 'path';
import * as fs from 'fs-extra';
import * as semver from "semver";
import * as extract from "extract-zip";
import * as Octokit from "@octokit/rest";
import * as util from "util";
import * as lockfile from "proper-lockfile";
import * as vscode from 'vscode';
import fetch from "node-fetch";
import AbortController from 'abort-controller';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions
} from 'vscode-languageclient/node';

enum LangServerBin {
    buildin,
    user,
    system,
}

const exec = util.promisify(require('child_process').exec);
const infoOutputChn = vscode.window.createOutputChannel('TreeCore Lang Client');
const traceOutputChn = vscode.window.createOutputChannel('TreeCore Lang Trace');

let client: LanguageClient;
const isWindows = process.platform === 'win32';
const langServerName = isWindows
    ? 'vhdl_ls-x86_64-pc-windows-msvc'
    : 'vhdl_ls-x86_64-unknown-linux-gnu';
// const langServerBinName = 'vhdl_ls';
const langServerBinName = 'treecore_ls';
let langServer: string;

export async function activate(ctx: vscode.ExtensionContext) {
    const langServerDir = ctx.asAbsolutePath(
        path.join('server', 'vhdl_ls')
    );

    infoOutputChn.appendLine(
        'Checking for language server executable in ' + langServerDir
    );

    let langServerVer = embeddedVersion(langServerDir);
    if (langServerVer == '0.0.0') {
        infoOutputChn.appendLine('No language server installed');
        vscode.window.showInformationMessage('Downloading language server...');
        await getLatestLanguageServer(60000, ctx);
        langServerVer = embeddedVersion(langServerDir);
    } else {
        infoOutputChn.appendLine('Found version ' + langServerVer);
    }

    // set found server path
    langServer = path.join(
        'server',
        'vhdl_ls',
        langServerVer,
        langServerName,
        'bin',
        langServerBinName + (isWindows ? '.exe' : '')
    );

    // get language server configuration and command to start server
    let workspace = vscode.workspace;
    let langServerBin = workspace.getConfiguration().get('tclc.languageServer');
    let lsBinary = langServerBin as keyof typeof LangServerBin;
    let serverOptions: ServerOptions;
    switch (lsBinary) {
        case 'buildin':
            serverOptions = getServerOptionsBuildin(ctx);
            infoOutputChn.appendLine('Using buildin language server');
            break;
        case 'user':
            serverOptions = getServerOptionsUser();
            infoOutputChn.appendLine('Using user specified language server');
            break;
        case 'system':
            serverOptions = getServerOptionsSystem();
            infoOutputChn.appendLine('Running language server from path');
            break;
        default:
            serverOptions = getServerOptionsBuildin(ctx);
            infoOutputChn.appendLine('Using buildin language server(default)');
            break;
    }

    // options to control the language client
    let clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'rvs' }],
        initializationOptions: vscode.workspace.getConfiguration('tclc'),
        traceOutputChannel: traceOutputChn,
    };
    if (workspace.workspaceFolders) {
        infoOutputChn.appendLine('[check workspace]' + workspace.workspaceFolders[0].uri.fsPath);
        clientOptions.synchronize = {
            fileEvents: workspace.createFileSystemWatcher(
                path.join(
                    workspace.workspaceFolders[0].uri.fsPath,
                    'vhdl_ls.toml'
                )
            ),
        };
    }

    // create the language client
    client = new LanguageClient(
        'vhdlls',
        'TreeCore Lang Server',
        serverOptions,
        clientOptions
    );

    // Start the client. This will also launch the server
    let langServerDisposable = client.start();
    ctx.subscriptions.push(langServerDisposable);
    ctx.subscriptions.push(
        vscode.commands.registerCommand('tclc.restart', async () => {
            const Msg = 'Restarting TreeCore LS';
            infoOutputChn.appendLine(Msg);
            vscode.window.showInformationMessage(Msg);
            await client.stop();
            langServerDisposable.dispose();
            langServerDisposable = client.start();
            ctx.subscriptions.push(langServerDisposable);
        })
    );

    infoOutputChn.appendLine('Language server started');
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

function embeddedVersion(langServerDir: string): string {
    try {
        return fs
            .readdirSync(langServerDir)
            .reduce((version: string, dir: string) => {
                if (semver.gt(dir, version)) {
                    fs.remove(path.join(langServerDir, version)).catch(
                        (err: any) => {
                            infoOutputChn.appendLine(err);
                        }
                    );
                    return dir;
                } else {
                    return version;
                }
            }, '0.0.0');
    } catch {
        return '0.0.0';
    }
}

function getServerOptionsBuildin(ctx: vscode.ExtensionContext) {
    let serverCommand = ctx.asAbsolutePath(langServer);
    let serverOptions: ServerOptions = {
        run: {
            command: serverCommand,
        },
        debug: {
            command: serverCommand,
        },
    };
    return serverOptions;
}

function getServerOptionsUser() {
    let serverCommand: string = vscode.workspace
        .getConfiguration()
        .get('tclc.languageServerUserPath')!; // HACK: err handle
    let serverOptions: ServerOptions = {
        run: {
            command: serverCommand,
        },
        debug: {
            command: serverCommand,
        },
    };
    return serverOptions;
}

function getServerOptionsSystem() {
    let serverCommand = langServerBinName;
    let serverOptions: ServerOptions = {
        run: {
            command: serverCommand,
        },
        debug: {
            command: serverCommand,
        },
    };
    return serverOptions;
}


const rustHdl = {
    owner: 'kraigher',
    repo: 'rust_hdl',
};

async function getLatestLanguageServer(
    timeoutMs: number,
    ctx: vscode.ExtensionContext
) {
    // get current and latest version
    const octokit = new Octokit({ userAgent: 'rust_hdl_vscode' });
    let latestRelease = await octokit.repos.getLatestRelease({
        owner: rustHdl.owner,
        repo: rustHdl.repo,
    });
    if (latestRelease.status != 200) {
        throw new Error('Status 200 return when getting latest release');
    }
    let current: string;
    if (langServer) {
        let { stdout } = await exec(
            `"${ctx.asAbsolutePath(langServer)}" --version`
        );
        current = semver.valid(semver.coerce(stdout.split(' ', 2)[1]))!; // HACK: err handle
    } else {
        current = '0.0.0';
    }

    let latest = semver.valid(semver.coerce(latestRelease.data.name))!; // HACK: err handle
    infoOutputChn.appendLine(`Current vhdl_ls version: ${current}`);
    infoOutputChn.appendLine(`Latest vhdl_ls version: ${latest}`);

    // download new version if available
    if (semver.prerelease(latest)) {
        infoOutputChn.appendLine('Latest version is pre-release, skipping');
    } else if (semver.lte(latest, current)) {
        infoOutputChn.appendLine('Language server is up-to-date');
    } else {
        const langServerAssetName = langServerName + '.zip';
        let browser_download_url = latestRelease.data.assets.filter(
            (asset) => asset.name == langServerAssetName
        )[0].browser_download_url;
        if (browser_download_url.length == 0) {
            throw new Error(
                `No asset with name ${langServerAssetName} in release.`
            );
        }

        infoOutputChn.appendLine('Fetching ' + browser_download_url);
        const abortController = new AbortController();
        const timeout = setTimeout(() => {
            abortController.abort();
        }, timeoutMs);
        let download = await fetch(browser_download_url, {
            signal: abortController.signal
        }).catch((err) => {
            infoOutputChn.appendLine(err);
            throw new Error(
                `Language server download timed out after ${timeoutMs.toFixed(
                    2
                )} seconds.`
            );
        });

        clearTimeout(timeout);
        if (download.status != 200) {
            throw new Error('Download returned status != 200');
        }
        const langServerAsset = ctx.asAbsolutePath(
            path.join('server', 'install', latest, langServerAssetName)
        );
        infoOutputChn.appendLine(`Writing ${langServerAsset}`);
        if (!fs.existsSync(path.dirname(langServerAsset))) {
            fs.mkdirSync(path.dirname(langServerAsset), {
                recursive: true,
            });
        }

        await new Promise<void>((resolve, reject) => {
            const dest = fs.createWriteStream(langServerAsset, {
                autoClose: true,
            });
            download.body.pipe(dest);
            dest.on('finish', () => {
                infoOutputChn.appendLine('Server download complete');
                resolve();
            });
            dest.on('error', (err: any) => {
                infoOutputChn.appendLine('Server download error');
                reject(err);
            });
        });

        await new Promise<void>((resolve, reject) => {
            const targetDir = ctx.asAbsolutePath(
                path.join('server', 'vhdl_ls', latest)
            );
            infoOutputChn.appendLine(
                `Extracting ${langServerAsset} to ${targetDir}`
            );
            if (!fs.existsSync(targetDir)) {
                fs.mkdirSync(targetDir, { recursive: true });
            }
            try {
                extract(langServerAsset, { dir: targetDir });
                infoOutputChn.appendLine('Server extracted');
                resolve();
            } catch (err) {
                infoOutputChn.appendLine('Error when extracting server');
                infoOutputChn.appendLine('Remove target');
                fs.removeSync(ctx.asAbsolutePath(path.join('server', 'install')));
                fs.removeSync(targetDir);
                reject(err);
            }
        });
    }
    return Promise.resolve();
}

function updateLanguageServer(ctx: vscode.ExtensionContext) {
    infoOutputChn.appendLine('Checking for updates...');
    lockfile
        .lock(ctx.asAbsolutePath('server'), {
            lockfilePath: ctx.asAbsolutePath(path.join('server', '.lock')),
        })
        .then((release: () => void) => {
            getLatestLanguageServer(60000, ctx)
                .catch((err) => {
                    infoOutputChn.appendLine(err);
                })
                .finally(() => {
                    infoOutputChn.appendLine('Language server update finished.');
                    return release();
                });
        });
}