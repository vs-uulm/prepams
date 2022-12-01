<template>
  <div>
    <v-card tile class="my-3" v-for="(study, i) in studies" :key="i">
      <v-list-item three-line>
        <v-list-item-content>
          <v-list-item-title>
            {{ study.name }}
          </v-list-item-title>
          <v-list-item-subtitle>{{ study.abstract }}</v-list-item-subtitle>
        </v-list-item-content>
        <v-list-item-action>
          <v-btn icon :to="`/study/${study.id}`">
            <v-icon>mdi-information-outline</v-icon>
          </v-btn>
        </v-list-item-action>
      </v-list-item>

      <v-list-item>
        <v-list-item-content>
          <v-list-item-subtitle class="pb-1">
            <v-btn small :to="`/study/${study.id}/participate`" :disabled="!$store.state.user || $store.state.user.role !== 'participant'" class="float-right" v-if="$store.state.user && !$store.state.user.participated[study.id]">
              <v-icon small left>mdi-clipboard-edit-outline</v-icon>
              Participate
            </v-btn>

            <v-chip color="green" text-color="white" small class="float-right" v-else-if="$store.state.user && $store.state.user.participated[study.id]">
              <v-icon small color="white" left>mdi-check-bold</v-icon>
              participated
            </v-chip>

            <study-info :study="study" />
          </v-list-item-subtitle>
        </v-list-item-content>
      </v-list-item>
    </v-card>

    <router-view />
  </div>
</template>

<script>
import axios from 'axios';

import StudyInfo from '@/components/StudyInfo';

export default {
  name: 'StudyOverview',

  components: {
    StudyInfo
  },

  data: () => {
    let resolver = null;
    const loaded = new Promise((resolve) => {
      resolver = resolve;
    });

    return {
      studies: [],
      loaded: loaded,
      resolver: resolver
    };
  },

  async mounted() {
    const res = await axios.get('/api/studies');
    this.studies = res.data;
    this.resolver();
  }
}
</script>
