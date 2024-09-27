import JSZip from 'jszip';
import UAParser from 'ua-parser-js';
import { saveAs } from 'file-saver';

window.init = async function(workload, headless = false) {
  window.workload = workload;
  window.headless = headless;
  window.filterRecord = new Function('job', 'workload', 'row', 'i', `return ${workload.filterRecord};`);

  const promises = [];
  const ready = new Promise((resolve) => promises.push(resolve));

  const worker = new Worker(new URL('./worker.js', import.meta.url));
  worker.addEventListener('message', (msg) => {
    if (typeof(msg.data) === 'string' && msg.data !== 'ready' && msg.data !== 'done') {
      document.querySelector('pre').innerText += `\n${msg.data}`;
    } else if (msg.data instanceof Error) {
      writeError(msg.data);
    } else {
      promises.shift()(msg.data);
    }
  });

  await ready;

  window.evaluation = new Proxy(worker, {
    get(target, job) {
      return (...args) => {
        return new Promise((resolve) => {
          promises.push(resolve);
          if (ArrayBuffer.isView(args[1])) {
            target.postMessage([job, args], [args[1].buffer]);
          } else {
            target.postMessage([job, args]);
          }
        });
      };
    }
  });

  await window.evaluation.setup(workload);
  return worker;
};

window.run = async function(name, input, jobs) {
  const times = [];
  for (const args of input) {
    let out = null;
    const time = {};

    for (const job of jobs) {
      const tmp = await window.evaluation[job](args, out);
      Object.assign(time, tmp[0]);
      out = tmp[1];
    }

    await window.tick();
    times.push(time);
  }

  await window.writeResponses(name, times.filter((row, i) => window.filterRecord(name, window.workload.name, row, i)));
};

window.runExperiments = async () => {
  try {
    for (const PHASE of ['WARMUP_', '']) {
      if (workload[`${PHASE}PARTICIPANTS`]?.length) {
        await window.run(`${PHASE}register`, workload[`${PHASE}PARTICIPANTS`], ['registerRequest', 'registerVerify', 'registerComplete']);
      }
      if (workload[`${PHASE}PARTICIPATIONS`]?.length) {
        await window.run(`${PHASE}participations`, workload[`${PHASE}PARTICIPATIONS`], ['participate', 'confirm', 'reward']);
      }
      if (workload[`${PHASE}PAYOUTS`]?.length) {
        await window.run(`${PHASE}payout`, workload[`${PHASE}PAYOUTS`], ['paddingRequest', 'paddingResponse', 'payoutRequest', 'payoutVerify']);
      }
    }
  } catch (e) {
    alert(e);
    console.error('[eval error]', e);
    window.writeError(e);
  }
}

setTimeout(async () => {
  if (!window.headless) {
    const res = await fetch('workloads/index.json');
    const workloads = (await res.json());

    if (navigator.clipboard) {
      const copy = document.createElement('button');
      copy.innerText = '📄';
      copy.style.position = 'fixed';
      copy.style.top = '2.52em';
      copy.style.right = '.5em';
      copy.style.fontSize = '200%';
      copy.addEventListener('click', () => {
        navigator.clipboard.writeText(pre.innerText);
      });
      document.body.appendChild(copy);
    }

    const btn = document.createElement('button');
    btn.style.fontSize = '200%';
    btn.style.margin = '1em';
    btn.innerHTML = 'Run All Experiments';
    document.body.appendChild(btn);

    try {
      const ua = UAParser();

      const deviceType = (ua.device.type === 'mobile' || ua.device.type === 'tablet') ? ua.device.type : 'laptop';
      const prefix = `${ua.os.name}-${ua.os.version}_${ua.browser.name}-${ua.browser.version}_${deviceType}`.replace(/[^a-zA-Z0-9-_]*/g, '');
      const zip = new JSZip();
      window.zip = zip.folder(prefix);

      const download = document.createElement('button');
      download.style.fontSize = '200%';
      download.style.margin = '1em';
      download.innerHTML = 'Download ZIP Archive';
      document.body.appendChild(download);

      download.addEventListener('click', async () => {
        download.disabled = true;
        try {
          const blob = await zip.generateAsync({ type: 'blob' });
          saveAs(blob, `${prefix}.zip`);
        } catch (e) {
          console.log(e);
          alert(e.toString());
        }
        download.disabled = false;
      });
    } catch (e) {
      console.log(e);
    }

    const label = document.createElement('label');
    label.innerText = 'WORKLOAD_SIZE:';
    label.style.fontSize = '200%';
    label.style.margin = '1em';
    label.for = 'workloadSize';
    document.body.appendChild(label);

    const select = document.createElement('select');
    select.id = 'workloadSize';
    select.style.fontSize = '200%';
    select.style.margin = '1em';
    select.innerHTML = '<option>PETS25_MINIMAL</option><option>PETS25_REDUCED</option><option>PETS25_FULL</option>';
    document.body.appendChild(select);

    const ul = document.createElement('ul');
    ul.innerHTML = workloads
      .map(e => `<li><a href="#" onclick="runWorkload('${e}')">run ${e}</a></li>`)
      .join('');
    document.body.appendChild(ul);

    const pre = document.createElement('pre');
    pre.style.backgroundColor = '#efefef';
    pre.style.border = '1px solid black';
    pre.style.maxWidth = 'calc(100% - 1em)';
    pre.style.margin = '.5em';
    pre.style.padding = '1em';
    pre.style.overflow = 'auto';
    pre.innerHTML = navigator.userAgent;
    document.body.appendChild(pre);

    btn.addEventListener('click', async () => {
      btn.remove();
      ul.remove();

      document.body.style.paddingTop = '4em';

      const progress = document.createElement('progress');
      progress.style.position = 'fixed';
      progress.style.top = '.5em';
      progress.style.left = '1em';
      progress.style.right = '1em';
      progress.style.width = 'calc(100% - 2em)';
      document.body.appendChild(progress);

      progress.max = (workloads.length || 0);
      progress.value = 0;

      let value = 0;
      try {
        for (const workload of workloads) {
          const start = Date.now();
          await window.runWorkload(workload);
          progress.value = ++value;
          pre.innerText += `\nFinished ${workload} in ${new Date(Date.now() - start).toTimeString()}\n`;
        }
      } catch (e) {
        alert(e);
        window.writeError(e);
      }

      progress.remove();
    });

    window.writeError = (e) => {
      pre.innerText += `\n\n${e?.stack || String(e)}`;
    };

    window.runWorkload = async (workload) => {
      btn?.remove?.();
      ul?.remove?.();

      pre.innerText += `\nExecuting workload ${workload}...\n\n`;

      const container = document.createElement('div');
      container.style.display = 'flex';
      container.style.position = 'fixed';
      container.style.top = '2em';
      container.style.left = '1em';
      container.style.right = '1em';
      document.body.appendChild(container);

      const title = document.createElement('div');
      title.innerText = workload;
      title.style.fontWeight = 'bold';
      title.style.width = '30%';
      container.appendChild(title);
      
      const progress = document.createElement('progress');
      progress.style.width = '70%';
      container.appendChild(progress);

      const res = await fetch(`workloads/${workload}.json`);
      let w = await res.json();
      w.name = workload;
      if (w.filterWorkload) {
        try {
          const filterWorkload = new Function(
            'workload',
            'WORKLOAD_SIZE',
            w.filterWorkload.slice(w.filterWorkload.indexOf('{') + 1, w.filterWorkload.lastIndexOf('}'))
          );

          w = filterWorkload(w, select.value);
        } catch (e) {
          console.log('[warn] exception while filtering workload', e);
        }
      }
      const worker = await window.init(w);

      progress.max = (window.workload.length || 0);
      progress.value = 0;
      let value = 0;

      window.tick = () => {
        progress.value = ++value;
      };

      window.writeResponses = async (name, times) => {
        if (times.length) {
          const header = Object.keys(times[0]);
          const content = `${header.join(',')}\n${times.map(e => header.map(h => e[h]).join(',')).join('\n')}`;
          pre.innerText += `\n\n${name}.csv\n${content}`;
          if (window.zip) {
            window.zip.file(`${workload}/${name}.csv`, content);
          }

          try {
            const res = await fetch(`/post?experiment=${workload}&file=${name}`, {
              headers: { 'content-type': 'text/plain' },
              method: 'POST',
              body: content
            });
            if (res.status !== 200) {
              throw new Error(res.statusText);
            }
            pre.innerText += '\n -> successfully uploaded results\n';
          } catch (e) {
            pre.innerText += `\n\nError uploading results: ${e.stack}`;
          }
        }
      };

      try {
        await window.runExperiments();
        worker.terminate();
      } catch (e) {
        alert(e);
        window.writeError(e);
      }

      container.remove();
    };
  }
}, 300); // xxx
