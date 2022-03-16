#include <iostream>
#include <chrono>
#include <unistd.h>
 
using namespace std;
 
// Main function to measure elapsed time of a C++ program
// using Chrono library
int main()
{
    auto start = chrono::steady_clock::now();
 
    // do some stuff here
    sleep(3);
 
    auto end = chrono::steady_clock::now();
 
    cout << "Elapsed time in nanoseconds: "
        << chrono::duration_cast<chrono::nanoseconds>(end - start).count()
        << " ns" << endl;
 
    cout << "Elapsed time in microseconds: "
        << chrono::duration_cast<chrono::microseconds>(end - start).count()
        << " µs" << endl;
 
    cout << "Elapsed time in milliseconds: "
        << chrono::duration_cast<chrono::milliseconds>(end - start).count()
        << " ms" << endl;
 
    cout << "Elapsed time in seconds: "
        << chrono::duration_cast<chrono::seconds>(end - start).count()
        << " sec";
 
    return 0;
}
