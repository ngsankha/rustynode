var timeoutRegistry = {};

var setTimeout = function(callback, timeout) {
  var timestamp = Date.now();
  timeoutRegistry[timestamp] = callback;
  _send(JSON.stringify({timestamp: timestamp, timeout: timeout}));
  //_print(JSON.stringify({timestamp: timestamp, timeout: timeout}));
}

var _recv = function(message) {
  timeoutRegistry[Number(message)]();
};

setTimeout(function() {
  _print('World');
}, 2000);
_print('Hello');
