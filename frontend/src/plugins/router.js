import Vue from 'vue'
import Router from 'vue-router'

import StudiesOverview from '@/views/StudiesOverview';
import StudiesList from '@/views/StudiesList';
import StudyForm from '@/views/StudyForm';
import StudyDialog from '@/views/StudyDialog';
import StudyParticipation from '@/views/StudyParticipation';

Vue.use(Router);

export default new Router({
  mode: 'history',
  base: process.env.BASE_URL,
  routes: [{
    path: '/',
    name: 'home',
    component: StudiesOverview,
    children: [{
      path: '/study/:id/:action?',
      component: StudyDialog
    }, {
      path: '/participation/:id',
      component: StudyParticipation
    }]
  }, {
    path: '/studies',
    name: 'studies',
    component: StudiesList,
    children: [{
      path: '/studies/:id',
      component: StudyDialog
    }]
  }, {
    path: '/studies/new',
    name: 'study',
    component: StudyForm
  }]
});
