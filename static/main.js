let app = new PIXI.Application(window.innerWidth, window.innerHeight, {
	backgroundColor: 0x1099bb,
	autoResize: true,
	resolution: devicePixelRatio
});
PIXI.settings.SCALE_MODE = PIXI.SCALE_MODES.NEAREST;
document.body.appendChild(app.view);

var graphics = new PIXI.Graphics();
graphics.lineStyle(2, 0x0000ff, 1);
graphics.beginFill(0xff700b, 1);
graphics.drawRect(0, 0, 800, 800);
app.stage.addChild(graphics);
graphics.x = window.innerWidth / 2 - 300;
graphics.y = window.innerHeight / 2 - 200;

const sprites = {};
const mobs = [];

let ws = new WebSocket("ws://" + window.location.host + "/ws/");
let opened = false;
let myid = 0;
let mypos = [300, 200];

ws.onopen = () => (opened = true);
ws.onclose = () => (opened = false);
ws.onmessage = function(e) {
	const m = JSON.parse(e.data);
	if (m.mobs) {
		m.mobs.forEach((m, i) => {
			if (!mobs[i]) {
				let sprite = PIXI.Sprite.fromImage("imgs/skeli.png");
				sprite.anchor.set(0.5);
				sprite.scale.x = 3;
				sprite.scale.y = 3;

				app.stage.addChild(sprite);
				mobs[i] = sprite;
			}
			mobs[i].x = m.pos[0] - mypos[0] + window.innerWidth / 2;
			mobs[i].y = m.pos[1] - mypos[1] + window.innerHeight / 2;
			mobs[i].alpha = m.health / 128;
		});
	}
	if (m.players) {
		m.players.forEach(p => {
			if (!sprites[p.id]) {
				let sprite = PIXI.Sprite.fromImage("imgs/bunny.png");
				sprite.anchor.set(0.5);
				sprite.scale.x = 4;
				sprite.scale.y = 4;

				app.stage.addChild(sprite);
				sprites[p.id] = sprite;
			}
			if (p.id == myid) mypos = p.pos;
			sprites[p.id].x = p.pos[0] - mypos[0] + window.innerWidth / 2;
			sprites[p.id].y = p.pos[1] - mypos[1] + window.innerHeight / 2;
			sprites[p.id].alpha = p.health / 128;
		});
	}
	graphics.x = window.innerWidth / 2 - mypos[0];
	graphics.y = window.innerHeight / 2 - mypos[1];
	if (m.id) myid = m.id;
};

function send(m) {
	if (opened) ws.send(JSON.stringify(m));
}
const keys = [0, 0];
window.addEventListener("keydown", e => {
	let keyCopy = keys.slice();
	if (e.keyCode == 68 || e.keyCode == 39) keys[0] = 1;
	if (e.keyCode == 65 || e.keyCode == 37) keys[0] = -1;
	if (e.keyCode == 83 || e.keyCode == 40) keys[1] = 1;
	if (e.keyCode == 87 || e.keyCode == 38) keys[1] = -1;
	if (keys[0] != keyCopy[0] || keys[1] != keyCopy[1]) send({ Keys: keys });
});
window.addEventListener("keyup", e => {
	if ((e.keyCode == 68 || e.keyCode == 39) && keys[0] == 1) keys[0] = 0;
	if ((e.keyCode == 65 || e.keyCode == 37) && keys[0] == -1) keys[0] = 0;
	if ((e.keyCode == 83 || e.keyCode == 40) && keys[1] == 1) keys[1] = 0;
	if ((e.keyCode == 87 || e.keyCode == 38) && keys[1] == -1) keys[1] = 0;
	send({ Keys: keys });
});
window.addEventListener("mousedown", e => {
	send({ Click: true });
});
window.addEventListener("mouseup", e => {
	send({ Click: false });
});
window.addEventListener("resize", e => {
	app.renderer.resize(window.innerWidth, window.innerHeight);
});
