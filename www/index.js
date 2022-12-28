import { World, Cell, ColorCalc } from "flower";
import { memory } from "flower/flower_bg";

window.mem = memory;


window.ColorCalc = ColorCalc;


const cv = document.getElementById('cv');


cv.width = window.innerWidth - 30;
cv.height = window.innerHeight - 100;

const ctx = cv.getContext('2d', { alpha: true });

const lower_boundary = Math.min(cv.height, cv.width);

const threshold = 5.0;

const big_radius = lower_boundary / 2 - lower_boundary / 50;
console.log(big_radius);
const dt = 1.0 / 60.0;

const nballs = 15000;
const w = World.new(nballs, big_radius - threshold, threshold, cv.width / 2, cv.height / 2, dt);

window.hex = function hex(n) {
    let s = n.toString(16);
    while (s.length < 6) s = '0' + s;
    return s;
}

window.prepCvs = function prepareCanvases() {

    const calc = ColorCalc.new(big_radius - threshold, nballs, dt);
    const color_nums = new Uint32Array(memory.buffer, calc.colors(), calc.color_count());

    const canvases = [...color_nums].map(coln => {
        const cv = document.createElement('canvas');
        cv.height = threshold * 2 + 2;
        cv.width = threshold * 2 + 2;
        
        const ctx = cv.getContext('2d', { alpha: true });


        ctx.fillStyle = '#' + hex(coln);


        ctx.beginPath();
        ctx.arc(cv.width / 2, cv.height / 2, threshold, 0.0, 2 * Math.PI);
        ctx.fill();

        return cv;
    });

    calc.free();


    return canvases;

}

const cvs = prepCvs();

let t = 0.0;




const backdrop = document.createElement('canvas')
backdrop.width = cv.width;
backdrop.height = cv.height;
const b_ctx = backdrop.getContext('2d', { alpha: false })
b_ctx.fillStyle = '#333333';
b_ctx.fillRect(0,0,cv.width, cv.height);


const color_indices = new Uint32Array(memory.buffer, w.indices(), nballs);


function draw() {

    ctx.drawImage(backdrop, 0, 0);

    ctx.lineWidth = 2.;


    w.prepare_colors(cvs.length);


    for (let i = nballs -1 ;i >= 0; --i) {
        ctx.drawImage(cvs[color_indices[i]], w.x(i) - threshold - 1, w.y(i) - threshold - 1);
    }
}

function loop() {
    w.simulate(dt);
    t += dt;
    draw();

    requestAnimationFrame(loop);

}

window.draw = draw;
window.w = w;
window.hex = hex;


requestAnimationFrame(loop);
