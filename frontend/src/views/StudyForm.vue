<template>
  <div>
    <h1 class="my-3">New Study</h1>

    <v-form ref="form" :disabled="loading" @submit.prevent="submit()">
      <v-card class="my-4">
        <v-card-title>
          Study Information
        </v-card-title>

        <v-card-text>
          <v-text-field label="Study Name" v-model="study.name" :rules="rules.name" :error-messages="errors.name"
            required />
          <v-text-field label="Study Abstract" counter="255" v-model="study.abstract" :error-messages="errors.abstract"
            :rules="rules.abstract" required />
          <v-textarea label="Detailed Description" counter="20000" v-model="study.description"
            :error-messages="errors.description" :rules="rules.description" required />
          <v-text-field label="Duration" v-model="study.duration" :rules="rules.duration"
            :error-messages="errors.duration" required />
        </v-card-text>
      </v-card>

      <v-card class="my-2">
        <v-card-title>
          Study Settings
        </v-card-title>

        <v-card-text>
          <v-text-field type="number" label="Rewarded Credits" v-model.number="study.reward"
            :error-messages="errors.reward" :rules="rules.reward" required persistent-hint
            hint="Define the amount of credits that will be transfered to the participant after participating in this study." />

          <v-card outlined class="mt-4">
            <v-card-text class="pa-4 pb-3">
              <div class="text-overline mb-0 mt-n2">
                Prerequisites
              </div>

              <v-list v-for="mode in ['qualifier', 'disqualifier']" :key="mode">
                <v-subheader>
                  {{ mode.charAt(0).toUpperCase() }}{{ mode.slice(1) }}
                  <small class="info--text ml-2 mb-1"><v-icon small color="info">mdi-information-outline</v-icon> {{ desc[mode] }}</small>
                </v-subheader>
                <v-list-item v-for="(id, i) in study[mode]" :key="i" :class="`${mode === 'qualifier' ? 'success' : 'error'} lighten-4`">
                  <v-list-item-icon>
                    <v-icon>{{ mode === 'qualifier' ? 'mdi-plus-thick' : 'mdi-cancel' }}</v-icon>
                  </v-list-item-icon>

                  <v-list-item-content>
                    <v-list-item-title>
                      {{ studies[id].name }}
                    </v-list-item-title>
                    <v-list-item-subtitle>
                      id: {{ id }}
                    </v-list-item-subtitle>
                  </v-list-item-content>

                  <v-list-item-action>
                    <v-btn icon @click="delPrerequisite(mode, id)"><v-icon>mdi-delete</v-icon></v-btn>
                  </v-list-item-action>
                </v-list-item>
                <v-alert class="ma-2" outlined dense type="error" v-if="errors[mode]">{{ errors[mode].join('') }}</v-alert>
              </v-list>

              <v-list>
                <v-subheader>
                  Attribute Constraints
                  <small class="info--text ml-2 mb-1">
                    <v-icon small color="info">mdi-information-outline</v-icon>
                    Participants have to satisfy ALL of these attribute constraints.
                  </small>
                </v-subheader>

                <v-list-item v-for="([e, t, params], i) in study.attributes" :key="i" class="info lighten-4">
                  <v-list-item-icon>
                    <v-icon>mdi-chart-box-plus-outline</v-icon>
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
                    <v-list-item-subtitle v-else-if="t === 'select'">
                      <span class="text-h6 pl-1 pr-2">∈</span>
                      <b>&lcub; {{ params.map(i => attributes[e][2][i]).join(', ') }} &rcub;</b>
                    </v-list-item-subtitle>
                  </v-list-item-content>

                  <v-list-item-action>
                    <v-btn icon @click="deleteAttributeConstraint(i)">
                      <v-icon>mdi-delete</v-icon>
                    </v-btn>
                  </v-list-item-action>
                </v-list-item>
              </v-list>
            </v-card-text>

            <v-card-actions>
              <v-spacer />

              <attribute-constraint @add="addAttributeConstraint" :attributes="attributes" v-if="attributes" />

              <v-menu offset-y v-for="mode in ['qualifier', 'disqualifier']" :key="mode">
                <template v-slot:activator="{ on, attrs }">
                  <v-btn :color="mode === 'qualifier' ? 'success' : 'error'" small v-bind="attrs" v-on="on" class="pr-4 ml-3">
                    <v-icon left small>{{ mode === 'qualifier' ? 'mdi-plus' : 'mdi-cancel' }}</v-icon>
                    Add {{ mode }}
                  </v-btn>
                </template>
                <v-list>
                  <v-list-item v-if="!studies || !Object.keys(studies).length" disabled>
                    <v-list-item-icon>
                      <v-icon>mdi-emoticon-confused-outline</v-icon>
                    </v-list-item-icon>
                    <v-list-item-content>
                      <v-list-item-title>No Data...</v-list-item-title>
                      <v-list-item-subtitle>You don't have access to any other studies</v-list-item-subtitle>
                    </v-list-item-content>
                  </v-list-item>
                  <v-list-item v-for="(study, index) in studies" :key="index" @click="addPrerequisite(mode, study.id)">
                    <v-list-item-content>
                      <v-list-item-title>{{ study.name }}</v-list-item-title>
                      <v-list-item-subtitle>id: {{ study.id }}</v-list-item-subtitle>
                    </v-list-item-content>
                  </v-list-item>
                </v-list>
              </v-menu>
            </v-card-actions>
          </v-card>

          <v-checkbox v-model="study.webBased" label="Is this study web-based?" :error-messages="errors.webBased"
            hide-details="" />
          <v-text-field v-model="study.studyURL" label="Study URL" :disabled="!study.webBased" :rules="rules.studyURL"
            :required="!study.webBased" :error-messages="errors.studyURL" />
        </v-card-text>

        <v-card-actions>
          <v-spacer />
          <v-btn type="submit" :loading="loading">
            <v-icon left>mdi-content-save-outline</v-icon>
            Create Study
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-form>
  </div>
</template>

<script>
import axios from 'axios';
import AttributeConstraint from '@/components/AttributeConstraint';

export default {
  name: 'StudyForm',

  components: {
    AttributeConstraint
  },

  data() {
    return {
      loading: false,
      attributes: null,
      studies: {},

      desc: {
        qualifier: 'Participants must have participated in ALL of these studies.',
        disqualifier: 'Participants must not have participated in ANY of these studies.'
      },

      study: {
        name: '',
        abstract: '',
        description: '',
        duration: '',
        reward: 0,
        qualifier: [],
        attributes: [],
        disqualifier: [],
        webBased: false,
        studyURL: '',
        details: null
      },

      rules: {
        name: [ v => !!v || 'Name is required' ],
        abstract: [ v => !!v || 'Abstract is required' ],
        description: [ v => !!v || 'Description is required' ],
        duration: [ v => !!v || 'Duration is required' ],
        reward: [ v => !!v || 'Rewarded credits are required' ],
        studyURL: [ v => !this.study.webBased || !!v || 'Study URL is required' ]
      },

      errors: {}
    };
  },

  async mounted() {
    if (!this.$store.state.user) {
      this.$router.push('/studies');
    }
    this.reload();
  },

  methods: {
    async submit() {
      this.errors = {};

      if (!this.$refs.form.validate()) {
        return;
      }

      try {
        this.loading = true;
        const id = await this.$store.dispatch('createStudy', this.study);

        if (id) {
          this.$router.push(`/studies/${id}`);
        }
      } catch (e) {
        if (Array.isArray(e?.response?.data?.errors)) {
          for (const error of e.response.data.errors) {
            this.errors[error.param.replace(/\[.*\]/g, '')] = [error.msg];
          }
        } else {
          this.$root.$handleError(e);
        }
      } finally {
        this.loading = false;
      }
    },

    addPrerequisite(mode, study) {
      if (!this.study[mode].includes(study)) {
        this.study[mode].push(study);
        this.study[mode].sort();
      }
    },

    addAttributeConstraint(e) {
      this.study.constraints.push(e);
      this.study.constraints.sort((a, b) => a[0] - b[0]);
    },

    deleteAttributeConstraint(index) {
      this.study.constraints.splice(index, 1);
    },

    delPrerequisite(mode, study) {
      const index = this.study[mode].indexOf(study);
      if (index >= 0) {
        this.study[mode].splice(index, 1);
      }
    },

    async reload() {
      const [studies, attributes] = await Promise.all([
        axios.get(`/api/studies/?id=${this.$store.state.user?.id}`),
        axios.get('/api/issuer/attributes')
      ]);

      this.attributes = attributes.data;
      for (const study of studies.data) {
        this.$set(this.studies, study.id, study);
      }
    },
  }
}
</script>
