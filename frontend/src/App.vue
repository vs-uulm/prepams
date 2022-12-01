<template>
  <v-app>
    <v-main>
      <v-container>
        <v-row>
          <v-col cols="12" xl="8" offset-xl="2">
            <v-card>
              <v-toolbar>
                <div class="d-flex align-center">
                  <v-img alt="PrePaMS" class="shrink mr-4" contain :src="require('@/assets/logo.svg')" transition="scale-transition" width="40" />
                </div>
                <h3 class="my-3 mr-6 pt-1 font-weight-bold" style="font-variant: small-caps">PrePaMS</h3>
                <v-btn text to="/">
                  <v-icon left>mdi-home-outline</v-icon>
                  Home
                </v-btn>
                <v-btn text to="/studies" :disabled="role !== 'organizer'">
                  <v-icon left>mdi-file-question-outline</v-icon>
                  My Studies
                </v-btn>

                <v-spacer />

                <v-btn @click="clearSessions()">
                  <v-icon left>mdi-bug</v-icon>
                  Clear Local Sessions
                </v-btn>
              </v-toolbar>

              <v-card-text>
                <v-row>
                  <v-col md="8" sm="6" cols="12">
                    <router-view />
                  </v-col>
                  <v-col md="4" sm="6" cols="12">
                    <user-authentication />
                  </v-col>
                </v-row>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </v-container>

      <v-dialog persistent v-model="dialog" :max-width="options.width" @keydown.esc="cancel">
          <v-card>
              <v-toolbar dark :color="options.type || options.color" dense text>
                  <v-toolbar-title class="white--text">
                    <v-icon v-if="options.type" left class="mt-n1">{{ icon[options.type] }}</v-icon>
                    {{ title }}
                  </v-toolbar-title>
              </v-toolbar>
              <v-card-text v-if="!options.prompt" v-show="!!message" :style="options.style" class="pt-5">{{ message }}</v-card-text>
              <v-card-text v-if="options.prompt" class="pt-5">
                  <v-textarea v-if="options.multiline" v-model="message" :label="options.label" :hint="options.hint" :persistent-hint="!!options.hint" autofocus rows="3" auto-grow outlined />
                  <v-text-field v-else v-model="message" :label="options.label" :hint="options.hint" :persistent-hint="!!options.hint" autofocus outlined />
              </v-card-text>
              <v-card-actions class="pt-5">
                  <v-spacer></v-spacer>
                  <v-btn v-if="!options.alert" text="text" @click.native="cancel">{{ options.cancelText }}</v-btn>
                  <v-btn v-if="!options.alert" text="text" @click.native="agree" :color="options.confirmColor">{{ options.confirmText }}</v-btn>
                  <v-btn v-if="options.alert" text="text" @click.native="cancel">Ok</v-btn>
              </v-card-actions>
          </v-card>
      </v-dialog>
    </v-main>
  </v-app>
</template>


<script>
import UserAuthentication from '@/components/UserAuthentication';

export default {
  data: () => ({
    dialog: false,
    resolve: null,
    reject: null,
    message: null,
    title: null,
    options: {},
    
    icon: {
      info: 'mdi-information-outline',
      warning: 'mdi-alert-outline',
      success: 'mdi-checkbox-marked-circle',
      error: 'mdi-alert-octagon-outline'
    }
  }),

  components: {
    UserAuthentication
  },

  created() {
    this.$root.$alert = (title, message = '', options = {}) => this.open(title, message, Object.assign({ alert: true }, options));
    this.$root.$confirm = (title, message = '', options = {}) => this.open(title, message, options);
    this.$root.$handleError = (e) => this.open('An error occurred!', e?.response?.data?.error || e?.message || '', { alert: true, type: 'error' });
    this.$root.$prompt = async(title, input = '', options = {}) => {
      const r = await this.open(title, input, Object.assign({ prompt: true }, options));
      return r ? this.message : r;
    };
  },

  methods: {
    open(title, message, options) {
      this.options = Object.assign({
        type: null,
        color: 'primary',
        width: 520,
        alert: false,
        prompt: false,
        multiline: false,
        hint: '',
        label: '',
        cancelText: 'Cancel',
        confirmText: 'Yes'
      }, options);

      this.options.style = {
        whiteSpace: 'pre-wrap',
        ...this.options.style
      };

      this.title = title;
      this.message = message;
      this.dialog = true;

      return new Promise((resolve, reject) => {
        this.resolve = resolve;
        this.reject = reject;
      });
    },

    agree() {
      this.resolve(true);
      this.dialog = false;
    },

    cancel() {
      this.resolve(false);
      this.dialog = false;
    },

    clearSessions() {
      Object.keys(localStorage)
        .filter(e => e.startsWith('participant:') || e.startsWith('organizer:'))
        .forEach(e => localStorage.removeItem(e));
      location.reload();
    }
  },

  computed: {
    role() {
      return this.$store.state.user?.role;
    }
  }
}
</script>

<style>
#app {
  background-color: #e5e3dd;
}

</style>
