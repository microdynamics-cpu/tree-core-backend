document.onkeydown = function (even) {
    // console.log('press');
    let jpCode = even.keyCode;
    document.getElementById('det').innerHTML = jpCode;
    let xhr = new XMLHttpRequest();
    xhr.open('POST', 'value', true);
    xhr.setRequestHeader("Content-type", "application/json");
    let obj = {
        "press": 1,
        "params": jpCode,
    };
    
    xhr.send(JSON.stringify(obj));
}

document.onkeyup = function(even) {
    // console.log('key up');
    document.getElementById('det').innerHTML = '0';
    let xhr = new XMLHttpRequest();
    xhr.open('POST', 'value', true);
    xhr.setRequestHeader("Content-type", "application/json");
    let obj = {
        "press": 0,
        "params": 0,
    };
    xhr.send(JSON.stringify(obj));
}

