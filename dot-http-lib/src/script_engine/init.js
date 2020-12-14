var client = {
  global: {}
};
client.global.set = function (key, value) {
  _snapshot[key] = value;
};
client.global.get = function (key) {
   if (_snapshot[key] != undefined) {
     return _snapshot[key];
   }
   if (_env[key] != undefined) {
     return _env[key];
   }
   return null;
};
