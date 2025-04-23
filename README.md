
Test WebSocket in browser console:
```
let socket = new WebSocket('ws://127.0.0.1:8000/socket');
socket.onmessage = (event) => {
    console.log(event.data);
};

socket.send('ls -l');

socket.close();
```