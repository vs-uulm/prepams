<template>
  <div>
    <v-btn icon class="float-right" to="/studies/new" color="primary" title="New Study">
      <v-icon>mdi-plus</v-icon>
    </v-btn>

    <h1 class="my-3">My Studies</h1>

    <v-card tile class="mt-10">
      <v-list-item three-line v-for="(study, i) in studies" :key="i">
        <v-list-item-content>
          <v-list-item-title>
            {{ study.name }}
          </v-list-item-title>
          <v-list-item-subtitle>{{ study.abstract }}</v-list-item-subtitle>
        </v-list-item-content>
        <v-list-item-action>
          <v-btn icon :to="`/studies/${study.id}`">
            <v-icon>mdi-clipboard-edit-outline</v-icon>
          </v-btn>
          <v-chip color="primary" outlined pill small>
            <v-icon left small>
              mdi-database
            </v-icon>
            {{ study.reward }} Credit{{ study.reward > 1 ? 's' : '' }}
          </v-chip>
        </v-list-item-action>
      </v-list-item>
    </v-card>

    <router-view />
  </div>
</template>

<script>
import axios from 'axios';

export default {
  name: 'StudiesList',

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

  mounted() {
    this.reload();
  },

  methods: {
    async reload() {
      if (this.$store.state.user) {
        if (this.$store.state.user.role !== 'organizer') {
          this.$router.push('/');
        }
        const res = await axios.get(`/api/studies?id=${this.$store.state.user.id}`);
        this.studies = res.data;
        this.resolver();
      }
    }
  },

  watch: {
    '$store.state.user.id': {
      handler() {
        this.reload();
      }
    }
  }
}
</script>
