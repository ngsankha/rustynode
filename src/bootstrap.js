var timeoutRegistry = {};

var setTimeout = function(callback, timeout) {
  var id = Math.floor(Math.random() * 999999) + 1;
  timeoutRegistry[id] = callback;
  _send('timeout', JSON.stringify({timestamp: id, timeout: timeout}));
};

var _recv = function(event, message) {
  switch (event) {
    case 'timeout':
      var key = Number(message);
      timeoutRegistry[key]();
      delete timeoutRegistry[key];
      break;
  }
};

var console = {
  log: _print
}

/////////////////////////////////////////////

setTimeout(function() {
  console.log('World');
  setTimeout(function() {
    console.log('I will always come later.');
  }, 1000);
}, 2000);
setTimeout(function() {
  console.log('But I come first!');
}, 1000);
console.log('Hello');
