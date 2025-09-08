import init from './pkg/web.js';

async function run() {
    try {
        await init();
    } catch(e) {
        console.error(e);
    }
}

run();