let app = new PIXI.Application(window.innerWidth, window.innerHeight, {
	backgroundColor: 0x1099bb
});
document.body.appendChild(app.view);
PIXI.settings.SCALE_MODE = PIXI.SCALE_MODES.NEAREST;

var sprites = {};

var ws = new WebSocket("ws://" + window.location.host + "/ws/");
function send(m) {
	ws.send(JSON.stringify(m));
}
ws.onopen = function() {};
var keys = {
	x: 0,
	y: 0
};
window.addEventListener("keydown", e => {
	if (e.keyCode == 39) keys.x = 1;
	if (e.keyCode == 37) keys.x = -1;
	if (e.keyCode == 40) keys.y = 1;
	if (e.keyCode == 38) keys.y = -1;

	send(keys);
});
window.addEventListener("keyup", e => {
	if (e.keyCode == 39 && keys.x == 1) keys.x = 0;
	if (e.keyCode == 37 && keys.x == -1) keys.x = 0;
	if (e.keyCode == 40 && keys.y == 1) keys.y = 0;
	if (e.keyCode == 38 && keys.y == -1) keys.y = 0;
	send(keys);
});
ws.onmessage = function(e) {
	var m = JSON.parse(e.data);
	if (m.players) {
		m.players.forEach(p => {
			if (!sprites[p.id]) {
				console.log("ok");
				var sprite = PIXI.Sprite.fromImage("imgs/bunny.png");
				sprite.anchor.set(0.5);
				sprite.scale.x = 5;
				sprite.scale.y = 5;

				app.stage.addChild(sprite);
				sprites[p.id] = sprite;
			}
			sprites[p.id].x = p.pos[0];
			sprites[p.id].y = p.pos[1];
		});
	}
};
