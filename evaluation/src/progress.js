const cliProgress = require('cli-progress');
const preset = cliProgress.Presets.rect.format.replace('%', '');

const bar = new cliProgress.MultiBar({
  clearOnComplete: false,
  noTTYOutput: true,
  hideCursor: false,
  barsize: 20,
}, cliProgress.Presets.rect);

bar.start = (title, task, max) => {
  const tmp = bar.create(max, 0, {}, {
    formatValue: (v, options, type) => {
      if (type === 'percentage') {
        return `${String(v).padStart(3)}% | ET: ${new Date(Date.now() - tmp.starttimer).toJSON().slice(11, 19).replace(/^(00?:?)*/, '').padStart(8)}s`;
      }
      return String(v).padStart(5);
    }
  });
  tmp.updateTitle = (title) => {
    tmp.options.title = `[${title}]`;
    tmp.options.format = `${tmp.options.title.padEnd(30)} ${String(tmp.options.task).padEnd(35)} ${preset}`;
  };
  tmp.updateTask = (task) => {
    tmp.options.task = task;
    tmp.options.format = `${tmp.options.title.padEnd(30)} ${String(tmp.options.task).padEnd(35)} ${preset}`;
  };
  tmp.starttimer = Date.now();
  tmp.options.task = task;
  tmp.updateTitle(title);
  tmp.stop = () => bar.remove(tmp);

  return tmp;
};

module.exports = bar;