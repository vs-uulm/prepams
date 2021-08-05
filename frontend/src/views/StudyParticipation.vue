<template>
  <v-dialog :value="participation" persistent max-width="400" scrollable>
    <v-card>
      <v-card-title class="text-h5 grey lighten-2">
        <v-icon left>mdi-qrcode-scan</v-icon>
        Participation

        <v-spacer />

        <v-btn icon @click="$router.push('/')" :disabled="loading">
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </v-card-title>

      <v-card-text class="pt-4">
        <v-list dense>
          <v-list-item v-if="study">
            <v-list-item-icon>
              <v-icon>mdi-clipboard-edit-outline</v-icon>
            </v-list-item-icon>
            <v-list-item-content>
              <v-list-item-title>Study: {{ study.name }}</v-list-item-title>
            </v-list-item-content>
          </v-list-item>
          <v-list-item v-else>
            <v-list-item-icon>
              <v-icon color="error">mdi-clipboard-alert-outline</v-icon>
            </v-list-item-icon>
            <v-list-item-content>
              <v-list-item-title class="error--text">Invalid Study</v-list-item-title>
            </v-list-item-content>
          </v-list-item>
          <v-list-item>
            <v-list-item-icon>
              <v-icon>mdi-signature-freehand</v-icon>
            </v-list-item-icon>
            <v-list-item-content>
              <v-list-item-title :class="`${valid ? 'success' : 'error'}--text`">
                Participation {{ valid ? 'valid' : 'invalid' }}
                <v-icon right small :color="valid ? 'success' : 'error'">{{ valid ? 'mdi-check-bold' : 'mdi-close-thick' }}</v-icon>
              </v-list-item-title>
            </v-list-item-content>
          </v-list-item>
        </v-list>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn :disabled="!study || !participation || !valid || loading" @click="complete()" :loading="loading" v-if="!completed">
          <v-icon left>mdi-database-export</v-icon>
          Complete Participation
        </v-btn>

        <v-chip color="green" text-color="white" label v-else>
          already completed
          <v-icon color="white" right>mdi-check-bold</v-icon>
        </v-chip>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script>
export default {
  name: 'StudyParticipation',

  data() {
    return {
      loading: false,
      completed: null,
      participation: null,
    };
  },

  mounted() {
    this.reload();
  },

  computed: {
    study() {
      if (this.participation?.id) {
        return this.$parent.studies.find(e => e.id === this.participation.id);
      }
      return null;
    },

    valid() {
      return this.participation?.valid;
    }
  },

  methods: {
    async reload() {
      if (this.$store.state.user?.role !== 'organizer') {
        return;
      }

      try {
        await this.$parent.loaded;
        this.participation = await this.$store.dispatch('checkParticipation', {
          id: this.$route.params.id,
          key: location.hash.slice(1)
        });
      } catch (e) {
        this.$root.$handleError(e);
        this.participationValid = false;
      }
    },

    async complete() {
      try {
        this.loading = true;
        if (await this.$root.$confirm('Complete Participation?', `Do you want to complete the participation and transfer ${this.study.reward} credits to the participant?`, { type: 'info' })) {
          await this.$store.dispatch('rewardParticipation', this.participation);
          this.completed = true;
        }
      } catch (e) {
        this.$root.$handleError(e);
      } finally {
        this.loading = false;
      }
    }
  },

  watch: {
    '$route.params.id': {
      handler() {
        this.reload();
      }
    },

    '$store.state.user': {
      handler() {
        this.reload();
      }
    }
  }
}
</script>
