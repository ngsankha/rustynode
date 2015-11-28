var _recv = function(event, id, data) {
  var key = Number(id);
  switch (event) {
    case 'timeout':
      callbackRegistry[key]();
      break;
    case 'readFile':
      callbackRegistry[key](data);
      break;
  }
  delete callbackRegistry[key];
};

var _generateID = function() {
  return Math.floor(Math.random() * 999999) + 1;
};

var console = {
  log: _print
}

var callbackRegistry = {};

var setTimeout = function(callback, timeout) {
  var id = _generateID();
  callbackRegistry[id] = callback;
  _send('timeout', JSON.stringify({id: id, timeout: timeout}));
};

var fs = {
  readFile: function(filename, callback) {
    var id = _generateID();
    callbackRegistry[id] = callback;
    _send('readFile', JSON.stringify({id: id, filename: filename}));
  }
};

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
fs.readFile('Cargo.toml', function(data) {
  console.log(data);
});
console.log('Hello');
