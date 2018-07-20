var ws = new WebSocket("ws://" + window.location.host + "/ws/");
var title = document.getElementById("title");
ws.onopen = function() {
	document.getElementById("messages").innerHTML = "";
	title.innerHTML = "Messages (Connected):";
};
fo.onsubmit = function(e) {
	e.preventDefault();
	var message = document.getElementById("message");

	if (fo.message.value !== "") {
		ws.send(fo.message.value);
		var text = document.createElement("li");
		text.style.background = "lightblue";
		text.innerHTML = "&#x2191; " + fo.message.value;
		document.getElementById("messages").appendChild(text);
		fo.message.value = "";
	}
};
ws.onmessage = function(e) {
	var text = document.createElement("li");
	text.innerHTML = "&#x2193; " + e.data;
	document.getElementById("messages").appendChild(text);
};
ws.onclose = function(e) {
	title.innerHTML = "Messages (Disconnected):";
	window.location.reload();
};
