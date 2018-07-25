let app = new PIXI.Application(window.innerWidth, window.innerHeight, {
	backgroundColor: 0x1099bb,
	autoResize: true,
	resolution: devicePixelRatio
});
PIXI.settings.SCALE_MODE = PIXI.SCALE_MODES.NEAREST;
document.body.appendChild(app.view);

const sprites = {};
const mobs = [];

let ws = new WebSocket("ws://" + window.location.host + "/ws/");
let opened = false;

ws.onopen = () => (opened = true);
ws.onclose = () => (opened = false);
ws.onmessage = function(e) {
	const m = JSON.parse(e.data);
	if (m.mobs) {
		m.mobs.forEach((m, i) => {
			if (!mobs[i]) {
				let sprite = PIXI.Sprite.fromImage("imgs/skeli.png");
				sprite.anchor.set(0.5);
				sprite.scale.x = 2;
				sprite.scale.y = 2;

				app.stage.addChild(sprite);
				mobs[i] = sprite;
			}
			mobs[i].x = m.pos[0];
			mobs[i].y = m.pos[1];
			mobs[i].alpha = m.health / 128;
		});
	}
	if (m.players) {
		m.players.forEach(p => {
			if (!sprites[p.id]) {
				let sprite = PIXI.Sprite.fromImage("imgs/bunny.png");
				sprite.anchor.set(0.5);
				sprite.scale.x = 5;
				sprite.scale.y = 5;

				app.stage.addChild(sprite);
				sprites[p.id] = sprite;
			}
			sprites[p.id].x = p.pos[0];
			sprites[p.id].y = p.pos[1];
			sprites[p.id].alpha = p.health / 128;
		});
	}
};

function send(m) {
	if (opened) ws.send(JSON.stringify(m));
}
const keys = [0, 0];
window.addEventListener("keydown", e => {
	let keyCopy = keys.slice();
	if (e.keyCode == 68) keys[0] = 1;
	if (e.keyCode == 65) keys[0] = -1;
	if (e.keyCode == 83) keys[1] = 1;
	if (e.keyCode == 87) keys[1] = -1;
	if (keys[0] != keyCopy[0] || keys[1] != keyCopy[1]) send(keys);
});
window.addEventListener("keyup", e => {
	if (e.keyCode == 68 && keys[0] == 1) keys[0] = 0;
	if (e.keyCode == 65 && keys[0] == -1) keys[0] = 0;
	if (e.keyCode == 83 && keys[1] == 1) keys[1] = 0;
	if (e.keyCode == 87 && keys[1] == -1) keys[1] = 0;
	send(keys);
});
window.addEventListener("resize", () => {
	app.renderer.resize(window.innerWidth, window.innerHeight);
});
