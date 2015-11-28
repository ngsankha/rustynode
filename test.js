
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
