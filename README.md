# Web Terminal

Run terminal in browser and execute commands on remote machine via WebSocket.

- Run server:
```
just serve
http://127.0.0.1:8000/terminal/?shell=sh
```

- Test WebSocket in browser console:
```
let socket = new WebSocket('ws://127.0.0.1:8000/socket/sh');
socket.onmessage = (event) => {
    console.log(event.data);
};

socket.send('ls -l');

socket.close();
```