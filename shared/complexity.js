const { exec } = require('node:child_process');
const { writeFile, appendFile } = require('node:fs/promises');

const run = (i) => new Promise((resolve, reject) => {
    console.time(`run${i}`);
    const cmd = `cargo run -q ${i}`;
    console.log(cmd);

    exec(cmd, (err, stdout, stderr) => {
        console.timeEnd(`run${i}`);
        if (err) {
            return reject(err);
        }

        const count = {
            proof: {
                mul: 0,
                add: 0,
            },
            verify: {
                mul: 0,
                add: 0,
            }
        };

        const lines = stdout.split('\n');
        let stage = 'proof';
        for (const line of lines) {
            if (line === 'count:add') {
                count[stage].add += 1;
            } else if (line === 'count:mul') {
                count[stage].mul += 1;
            } else if (line === 'count:verify') {
                stage = 'verify';
            } else if (line) {
                console.log('wtf', line);
            }
        }
        resolve(count);
    });
});


(async () => {
    const results = [];

    await writeFile('complexity.csv', 'input_length,proof_mul,proof_add,verify_mul,verify_add\n');

    for (let i = 0; i < 1000; i += 1) {
        const res = await run(i);
        res.input_length = i;
        results.push(res);
        await appendFile(
            'complexity.csv',
            `${res.input_length},${res.proof.mul},${res.proof.add},${res.verify.mul},${res.verify.add}\n`
        );
    }

    console.table(results);
})();
