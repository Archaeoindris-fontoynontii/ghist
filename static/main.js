let app = new PIXI.Application(window.innerWidth, window.innerHeight, {
	backgroundColor: 0xf1ddae,
	autoResize: true,
	resolution: devicePixelRatio
});
PIXI.settings.SCALE_MODE = PIXI.SCALE_MODES.NEAREST;
document.body.appendChild(app.view);
app.stage.position.x = window.innerWidth / 2;
app.stage.position.y = window.innerHeight / 2;
app.stage.pivot.x = 400;
app.stage.pivot.y = 400;

var basetex = PIXI.BaseTexture.fromImage("imgs/desert.png");
function createWorld() {
	var texture = new PIXI.Texture(basetex, new PIXI.Rectangle(96, 0, 32, 32));
	let sand = new PIXI.extras.TilingSprite(texture, 800, 800);
	sand.tileScale.x = 2;
	sand.tileScale.y = 2;
	app.stage.addChild(sand);
}
function topBorder() {
	var texture = new PIXI.Texture(basetex, new PIXI.Rectangle(96, 96, 32, 32));
	let sand = new PIXI.extras.TilingSprite(texture, 800, 64);
	sand.tileScale.x = 2;
	sand.tileScale.y = 2;
	app.stage.addChild(sand);
}
createWorld();
topBorder();

const sprites = {};
const mobs = [];

var loaded = false;
var frames = [];
app.loader.add("imgs/testing.json").load(_ => {
	for (var i = 0; i < 33; i++) {
		frames.push(PIXI.Texture.fromFrame("00" + (i > 9 ? i : "0" + i) + ".png"));
	}
	loaded = true;
	for (var k in sprites) {
		sprites[k].texture = frames[0];
	}
	for (var k in mobs) {
		mobs[k].texture = frames[0];
	}
});

let ws = new WebSocket("ws://" + window.location.host + "/ws/");
let opened = false;
let myid = 0;
let mypos = [300, 200];

document.getElementById("username").focus();
document.getElementById("username").addEventListener("keydown", e => {
	if (e.keyCode == 13 && opened) {
		send({ Name: document.getElementById("username").value });
		document.getElementById("login").style.display = "none";
	}
});
ws.addEventListener("open", () => {
	opened = true;
	document.getElementById("status").innerText = "Press enter to play";
});
ws.addEventListener("close", () => (opened = false));
ws.addEventListener("message", e => {
	const m = JSON.parse(e.data);
	if (m.id) myid = m.id;
	if (m.mobs) {
		m.mobs.forEach((m, i) => {
			if (!mobs[i]) {
				let sprite = PIXI.Sprite.from(loaded ? "0001.png" : PIXI.Texture.EMPTY);
				sprite.anchor.set(0.5);
				sprite.scale.x = 3;
				sprite.scale.y = 3;

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
				let sprite = PIXI.Sprite.from(loaded ? "0005.png" : PIXI.Texture.WHITE);
				sprite.anchor.set(0.5);
				sprite.scale.x = 4;
				sprite.scale.y = 4;

				app.stage.addChild(sprite);
				sprites[p.id] = sprite;
			}
			if (p.id == myid) {
				app.stage.pivot.x = p.pos[0];
				app.stage.pivot.y = p.pos[1];
			}
			sprites[p.id].x = p.pos[0];
			sprites[p.id].y = p.pos[1];
			sprites[p.id].alpha = p.health / 128;
		});
	}
});

function send(m) {
	if (opened) ws.send(JSON.stringify(m));
}
const keys = [0, 0];
window.addEventListener("mousemove", e => {
	var a = Math.atan2(
		e.clientX - window.innerWidth / 2,
		e.clientY - window.innerHeight / 2
	);
	var angle = (a / Math.PI) * 16;
	if (angle < 0) angle += 32;
	if (sprites[myid]) sprites[myid].texture = frames[Math.floor(angle)];
});
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
	app.stage.position.x = window.innerWidth / 2;
	app.stage.position.y = window.innerHeight / 2;
});
