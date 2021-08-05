import Vue from 'vue';
import App from './App';

import vuetify from './plugins/vuetify';
import router from './plugins/router';
import store from './plugins/store';
import '@fontsource/roboto';
import '@mdi/font/css/materialdesignicons.css';

Vue.config.productionTip = false;

import axios from 'axios';
axios.defaults.baseURL = process.env['BASE_URL'];

new Vue({
  vuetify,
  router,
  store,
  render: h => h(App)
}).$mount('#app');
