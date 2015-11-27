var timeoutRegistry = {};

var setTimeout = function(callback, timeout) {
  var id = Math.floor(Math.random() * 999999) + 1;
  timeoutRegistry[id] = callback;
  _send(JSON.stringify({timestamp: id, timeout: timeout}));
}

var _recv = function(message) {
  var key = Number(message);
  timeoutRegistry[key]();
  delete timeoutRegistry[key];
};

/////////////////////////////////////////////

setTimeout(function() {
  _print('World');
  setTimeout(function() {
    _print('I will always come later.');
  }, 1000);
}, 2000);
setTimeout(function() {
  _print('But I come first!');
}, 1000);
_print('Hello');
