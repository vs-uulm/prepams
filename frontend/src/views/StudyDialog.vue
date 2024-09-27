<template>
  <v-dialog :value="true" persistent max-width="700" scrollable>
    <v-card v-if="study">
      <v-card-title class="text-h5 grey lighten-2">
        <v-icon left>mdi-clipboard-edit-outline</v-icon>

        <span style="max-width: 89%; text-overflow: ellipsis; overflow: hidden; white-space: nowrap;"
          :title="study.name">{{ study.name }}</span>

        <v-spacer />

        <v-btn icon @click="close()" :disabled="loading || (participationCode && !participationCodeSaved && !urlCopied)">
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </v-card-title>

      <v-card-subtitle class="grey lighten-2 py-3 mb-5">
        Created by: {{ study.owner }}
      </v-card-subtitle>

      <v-card-text v-if="!participationCode">
        <study-info :study="study" />

        <v-overlay :value="waiting" style="cursor: wait;">
          <v-card width="min(95vw,400px)">
            <v-card-title class="info--text">
              <v-icon left color="info">mdi-timer-sand</v-icon>
              Waiting for completion...
            </v-card-title>
            <v-card-text class="text-center">
              <v-alert outlined type="info" dense class="text-left">
                Complete the survey using the external study software to continue.
              </v-alert>

              <v-progress-circular indeterminate :size="100" color="info" />
            </v-card-text>
          </v-card>
        </v-overlay>

        <v-card outlined class="mt-4">
          <v-card-text class="pa-4 pb-3">
            <div class="text-overline mb-0 mt-n2">
              Abstract
            </div>

            <div>{{ study.abstract }}</div>
          </v-card-text>
        </v-card>

        <v-card outlined class="mt-4">
          <v-card-text class="pa-4 pb-3">
            <div class="text-overline mb-0 mt-n2">
              Description
            </div>

            <div style="white-space: pre-wrap;">{{ study.description }}</div>
          </v-card-text>
        </v-card>

        <v-card outlined class="mt-4" v-if="qualifier.length > 0 || disqualifier.length > 0 || study.constraints.length > 0">
          <v-card-text class="pa-4 pb-3">
            <div class="text-overline mb-0 mt-n2">
              Prerequisites
            </div>

            <v-list dense class="py-0">
              <v-list-item v-for="(study, i) in qualifier" :key="`q${i}`" :to="`/study/${study.id}`">
                <v-list-item-icon>
                  <v-tooltip bottom>
                    <template v-slot:activator="{ on, attrs }">
                      <v-icon color="success" v-bind="attrs" v-on="on">
                        mdi-plus-thick
                      </v-icon>
                    </template>
                    <span>Qualifier: Prior participation in this study required.</span>
                  </v-tooltip>
                </v-list-item-icon>

                <v-list-item-content>
                  <v-list-item-title>
                    {{ study.name }}
                  </v-list-item-title>
                  <v-list-item-subtitle>
                    id: {{ study.id }}
                  </v-list-item-subtitle>
                </v-list-item-content>
              </v-list-item>

              <v-list-item v-for="(study, i) in disqualifier" :key="`d${i}`" :to="`/study/${study.id}`">
                <v-list-item-icon>
                  <v-tooltip bottom>
                    <template v-slot:activator="{ on, attrs }">
                      <v-icon color="error" v-bind="attrs" v-on="on">
                        mdi-cancel
                      </v-icon>
                    </template>
                    <span>Disqualifier: Prior participation in this study disqualifies from participation.</span>
                  </v-tooltip>
                </v-list-item-icon>

                <v-list-item-content>
                  <v-list-item-title>
                    {{ study.name }}
                  </v-list-item-title>
                  <v-list-item-subtitle>
                    id: {{ study.id }}
                  </v-list-item-subtitle>
                </v-list-item-content>
              </v-list-item>

              <v-list-item v-for="([e, t, params], i) in study.constraints" :key="`a${i}`">
                <v-list-item-icon>
                  <v-tooltip bottom>
                    <template v-slot:activator="{ on, attrs }">
                      <v-icon color="info" v-bind="attrs" v-on="on">
                        mdi-information-outline
                      </v-icon>
                    </template>
                    <span>Attribute Constraint: Participants have to satisfy ALL of these attribute constraints.</span>
                  </v-tooltip>
                </v-list-item-icon>

                <v-list-item-content>
                  <v-list-item-title>
                    {{ attributes[e][0] }}
                  </v-list-item-title>
                  <v-list-item-subtitle v-if="t === 'number'">
                    <b>{{ params[0] }}</b>
                    <v-icon class="mx-2" small>mdi-less-than-or-equal</v-icon>
                    <b>{{ attributes[e][0] }}</b>
                    <v-icon class="mx-2" small>mdi-less-than-or-equal</v-icon>
                    <b>{{ params[1] }}</b>
                  </v-list-item-subtitle>
                  <v-list-item-subtitle v-else-if="t === 'select'" class="mt-n2">
                    <span class="text-subtitle-1 pl-1 pr-2">∈</span>
                    <b>&lcub; {{ params.map(i => attributes[e][2][i]).join(', ') }} &rcub;</b>
                  </v-list-item-subtitle>
                </v-list-item-content>
              </v-list-item>
            </v-list>
          </v-card-text>
        </v-card>
      </v-card-text>

      <v-card-text v-else>
        <v-row>
          <v-col cols="4">
            <v-img :src="participationCode" v-if="participationCode" contain max-height="200" />
          </v-col>
          <v-col cols="8">
            <v-alert type="info" outlined>
              Present this participation code when participating in the study.
              The embedded information allows the study organizer to validate your participation and transfer participation
              rewards to you, but does not reveal any personal identifiable information.
            </v-alert>
            
            <v-alert :type="!participationCodeSaved ? 'warning' : 'success'" class="mt-4" outlined>
              Your participation code will not be saved.
              Download it using the button below.

              <v-btn color="warning" :href="participationFile" :download="`PrePaMS-ParticipationCode-${study.id}.png`"
                @click="participationCodeSaved = true" class="float-right mt-4">
                <v-icon left>mdi-file-pdf-outline</v-icon>
                Download Participation Code
              </v-btn>
            </v-alert>
          </v-col>
        </v-row>

        <v-alert type="info" icon="mdi-school" outlined>
          In this prototype demo, you can also navigate to the following participation URL in the study
          organizer's browser window.

          <v-text-field solo :value="participationURL" readonly style="font-family: monospace; font-size: 85%;" class="mt-2"
            hide-details @click="$event.target.select()" append-icon="mdi-content-copy" @click:append="copyCode()" />

          <v-snackbar v-model="urlCopied">
            URL copied to clipboard!
            <template v-slot:action="{ attrs }">
              <v-btn icon v-bind="attrs" @click="urlCopied = false">
                <v-icon>mdi-close</v-icon>
              </v-btn>
            </template>
          </v-snackbar>
        </v-alert>
      </v-card-text>

      <v-divider />

      <v-card-actions v-if="!participationCode && $store.state.user && $store.state.user.role === 'participant'">
        <v-spacer />

        <v-chip color="green" text-color="white" class="float-right" v-if="$store.state.user.participated && $store.state.user.participated.has(study.id)">
          <v-icon color="white" left>mdi-check-bold</v-icon>
          participated
        </v-chip>
        <v-btn color="primary" @click="participate" v-else :disabled="!$store.state.user || $store.state.user.role !== 'participant' || loading" :loading="loading">
          <v-icon small left>mdi-clipboard-edit-outline</v-icon>
          {{ $store.state.user ? 'Participate' : 'Log In to Participate' }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script>
import axios from 'axios';
import StudyInfo from '@/components/StudyInfo';

export default {
  name: 'StudyDialog',

  components: {
    StudyInfo
  },

  data() {
    return {
      loading: false,
      waiting: false,
      attributes: null,

      qualifier: [],
      disqualifier: [],

      urlCopied: false,
      participationURL: null,
      participationCode: null,
      participationFile: null,
      participationTimer: null,
      participationCodeSaved: false,

      study: {
        name: '',
        abstract: '',
        description: '',
        duration: '',
        reward: 0,
        qualifier: [],
        disqualifier: [],
        constraints: [],
        webBased: false,
        studyURL: '',
        details: null
      }
    };
  },

  async mounted() {
    await this.$parent.loaded;
    await this.reload();
  },

  beforeDestroy() {
    if (this.participationTimer) {
      clearInterval(this.participationTimer);
    }
  },

  methods: {
    async reload() {
      await this.$parent.loaded;

      const req = await axios.get('/api/issuer/attributes');
      this.attributes = req.data;

      const study = this.$parent.studies.find(e => e.id === this.$route.params.id);
      if (!study) {
        await this.$root.$alert('Error: Study Not Found!', '\n', { type: 'error' });
        this.$router.push('/');
        return;
      }

      this.study = study;
      this.qualifier = this.study.qualifier.map(s => this.$parent.studies.find(e => e.id === s));
      this.disqualifier = this.study.disqualifier.map(s => this.$parent.studies.find(e => e.id === s));

      if (this.$route.params.action === 'participate') {
        await this.participate();
      }

      if (this.$store.state.user?.role === 'organizer' && this.$store.state.user?.id === study.owner) {
        const res = await axios.get(`/api/rewards/${this.study.id}`);
        this.$set(study, 'participations', res.data.transactions);
      }
    },

    async participate() {
      this.loading = true;
      const warnings = [];

      try {
        if (this.qualifier.length > 0) {
          warnings.push(`Your participation in this study will reveal to the organizer that you participated in the following studies:\n${this.qualifier.map(e => `   • ${e.name}`).join('\n')}`);
        }

        if (this.disqualifier.length > 0) {
          warnings.push(`Your participation in this study will reveal to the organizer that you have not participated in the following studies:\n${this.disqualifier.map(e => `   • ${e.name}`).join('\n')}`);
        }

        if (!await this.$root.$confirm('Do you want to participate in this study?', warnings.join('\n\n'), { type: 'warning' })) {
          return;
        }

        const {
          participation,
          participationURL,
          participationCode,
          participationFile
        } = await this.$store.dispatch('participate', this.study);

        if (this.study.webBased) {
          if (await this.$root.$confirm('Redirecting to Web-based Study...', [
            'You will be redirected to the following URL:',
            this.study.studyURL,
            '',
            'Your participation data will be transfered during this process as well, this does not contain any personal identifiable information.'
          ].join('\n'), { type: 'info' })) {
            this.waiting = true;

            let target;
            window.addEventListener('message', async (e) => {
              console.log('on:message', e.data)
              if (e.data === 'ready' && target) {
                target.postMessage({
                  type: 'prepams-participation',
                  participation: {
                    context: `${this.study.name} (${this.study.reward} credits)`,
                    href: `${location.origin}/study/${this.study.id}`,
                    code: participationCode,
                    data: participation
                  }
                }, new URL(this.study.studyURL).origin);
              } else if (e.data?.type === 'prepams-confirmation') {
                const buf = await fetch(`data:application/octet-stream;base64,${e.data.confirmation}`);
                const confirmedParticipation = new Uint8Array(await buf.arrayBuffer());
                await axios.post(`/api/rewards`, confirmedParticipation, {
                  headers: { 'Content-Type': 'application/octet-stream' },
                });
                this.waiting = false;
                await this.$store.dispatch('refreshBalance');
                if (this.$store.state.user.participated?.has?.(this.study.id)) {
                  target.postMessage({ type: 'prepams-completed' }, new URL(this.study.studyURL).origin);
                }
                this.reload();
                if (this.$route.fullPath !== `/study/${this.study.id}`) {
                  this.$router.push(`/study/${this.study.id}`);
                }
              }
            });
            target = window.open(this.study.studyURL);
          }
        } else {
          this.participationCode = participationCode;
          this.participationFile = participationFile;
          this.participationURL = participationURL;
          this.participationTimer = setInterval(async () => {
            // check participation
            await this.$store.dispatch('refreshBalance');
            if (this.$store.state.user.participated?.has?.(this.study.id)) {
              clearInterval(this.participationTimer);
              this.participationCodeSaved = true;
              this.participationTimer = null;
              this.participationCode = null;
              this.participationFile = null;
              this.participationURL = null;
              this.reload();
              if (this.$route.fullPath !== `/study/${this.study.id}`) {
                this.$router.push(`/study/${this.study.id}`);
              }
            }
          }, 5000);
        }
      } catch (e) {
        this.$root.$handleError(e);
      } finally {
        this.loading = false;
      }
    },

    async copyCode() {
      await navigator.clipboard.writeText(this.participationURL);
      this.urlCopied = true;
      setTimeout(() => {
        this.urlCopied = false;
      }, 5000);
    },

    close() {
      this.$router.push(this.$route.fullPath.startsWith('/studies') ? '/studies' : '/');
    }
  },

  watch: {
    '$route.params.id': {
      handler() {
        this.reload();
      }
    },

    '$route.params.action': {
      handler() {
        this.reload();
      }
    }
  }
}
</script>
