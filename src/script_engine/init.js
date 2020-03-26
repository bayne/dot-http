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

function readCookie(response, variable_name) {
	let set_cookie = response.headers['set-cookie'];
	let pos = set_cookie.indexOf(';');
	client.global.set(variable_name, set_cookie.substring(0,pos));
}

function assertEquals(first, second, message)  {
	if (first != second) {
		let gen_message = "assertion equals failed first: "+ first + " second : "+ second;
		if (message) {
			gen_message += message;
		}
		throw gen_message;
	}
}

