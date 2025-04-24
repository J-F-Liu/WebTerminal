import './style.css'

// const webSocket = new WebSocket('ws://127.0.0.1:8000/socket/sh');
const webSocket = new WebSocket('ws://' + window.location.host + '/socket/sh');

let length = 0;
let command = '';
let textarea = document.getElementById('textarea');

webSocket.onmessage = function (event) {
  textarea.value += event.data + '\n> ';
  length = textarea.value.length;
  textarea.scrollTop = textarea.scrollHeight;
}

textarea.onkeydown = function (event) {
  if (event.key === 'Enter') {
    command = textarea.value.substring(length).trim();
    if (command.length > 0) {
      if (command === 'clear') {
        textarea.value = '> ';
        length = textarea.value.length;
        event.preventDefault();
      }
      else {
        webSocket.send(command);
      }
    } else {
      textarea.value += '\n> ';
      length = textarea.value.length;
      event.preventDefault();
    }
  } else if (event.key === 'ArrowUp') {
    textarea.value += command;
    command = '';
    event.preventDefault();
  }
}