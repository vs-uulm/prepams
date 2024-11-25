<template>
  <v-app>
    <v-main>
      <v-container>
        <v-row>
          <v-col cols="12" xl="8" offset-xl="2">
            <v-card>
              <v-toolbar>
                <div class="d-flex align-center" @click="about = true" :style="{ cursor: ABOUT ? 'pointer' : null }">
                  <v-img alt="PrePaMS Logo" class="shrink mr-4" contain :src="require('@/assets/logo.svg')" transition="scale-transition" width="40" />
                </div>
                <h3 class="my-3 mr-6 pt-1 font-weight-bold" :style="{ fontVariant: 'small-caps', cursor: ABOUT ? 'pointer' : null }" @click="about = true">PrePaMS</h3>
                <div class="d-flex d-md-none flex-grow-1 justify-end">
                  <v-menu offset-y>
                    <template #activator="{ on, attrs }">
                      <v-btn icon color="primary" dark v-bind="attrs" v-on="on">
                        <v-icon>mdi-menu</v-icon>
                      </v-btn>
                    </template>
                    <v-list>
                      <v-list-item to="/">
                        <v-list-item-icon>
                          <v-icon>mdi-home-outline</v-icon>
                        </v-list-item-icon>
                        <v-list-item-title>Home</v-list-item-title>
                      </v-list-item>
                      <v-list-item to="/studies" :disabled="role !== 'organizer'">
                        <v-list-item-icon>
                          <v-icon>mdi-file-question-outline</v-icon>
                        </v-list-item-icon>
                        <v-list-item-title>My Studies</v-list-item-title>
                      </v-list-item>

                      <v-divider />

                      <v-list-item to="/payouts" class="warning--text">
                        <v-list-item-icon>
                          <v-icon color="warning">mdi-bug</v-icon>
                        </v-list-item-icon>
                        <v-list-item-title>Payouts Log</v-list-item-title>
                      </v-list-item>

                      <v-list-item @click="clearSessions()">
                        <v-list-item-icon>
                          <v-icon>mdi-bug</v-icon>
                        </v-list-item-icon>
                        <v-list-item-title>Clear Local Sessions</v-list-item-title>
                      </v-list-item>

                      <v-list-item @click="about = true" class="info--text" v-if="ABOUT">
                        <v-list-item-icon>
                          <v-icon color="info">mdi-information-variant</v-icon>
                        </v-list-item-icon>
                        <v-list-item-title>About PrePaMS</v-list-item-title>
                      </v-list-item>
                    </v-list>
                  </v-menu>
                </div>
                
                <div class="d-none d-md-flex flex-grow-1">
                  <v-btn text to="/">
                    <v-icon left>mdi-home-outline</v-icon>
                    Home
                  </v-btn>
                  <v-btn text to="/studies" :disabled="role !== 'organizer'">
                    <v-icon left>mdi-file-question-outline</v-icon>
                    My Studies
                  </v-btn>

                  <v-spacer />

                  <v-btn to="/payouts" color="warning" class="mr-3">
                    <v-icon left>mdi-bug</v-icon>
                    Payouts Log
                  </v-btn>

                  <v-btn @click="clearSessions()">
                    <v-icon left>mdi-bug</v-icon>
                    Clear Local Sessions
                  </v-btn>

                  <v-btn @click="about = true" v-if="ABOUT" color="info" class="ml-3">
                    <v-icon left>mdi-information-variant</v-icon>
                    About
                  </v-btn>
                </div>
              </v-toolbar>

              <v-alert v-model="about" color="info" dismissible v-if="ABOUT">
                <template #prepend><span class="mr-2 mb-auto" v-html="ABOUT_ICON"></span></template>
                <div v-html="ABOUT"></div>
                <template #close="{ toggle }">
                  <v-btn rounded icon small @click="toggle" class="mb-auto mr-n1 mt-n1"><v-icon>mdi-close-circle</v-icon></v-btn>
                </template>
              </v-alert>

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
    <v-footer padless>
      <v-col class="text-center" cols="12">
        &copy; {{ new Date().getFullYear() }}
        <strong class="mr-3 ml-1"><a href="https://www.uni-ulm.de/en/in/vs/" target="_blank">Institute of Distributed Systems, Ulm University</a></strong>
        |
        <v-btn text small href="https://github.com/vs-uulm/prepams" target="_blank"><v-icon left>mdi-github</v-icon>GitHub</v-btn>
        <v-btn text small href="https://doi.org/10.56553/popets-2025-0034" target="_blank"><v-icon left>mdi-file-document-outline</v-icon>Publication</v-btn>
        <template v-if="FOOTER">
          | <span v-html="FOOTER"></span>
        </template>
      </v-col>
    </v-footer>
  </v-app>
</template>


<script>
import UserAuthentication from '@/components/UserAuthentication';

export default {
  data: () => ({
    about: localStorage.getItem('about-dismissed') !== 'true',
    drawer: false,
    dialog: false,
    resolve: null,
    promise: null,
    reject: null,
    message: null,
    title: null,
    options: {},
    
    icon: {
      info: 'mdi-information-outline',
      warning: 'mdi-alert-outline',
      success: 'mdi-checkbox-marked-circle',
      error: 'mdi-alert-octagon-outline'
    },

    ABOUT: process.env['VUE_APP_ABOUT'] || '',
    ABOUT_ICON: process.env['VUE_APP_ABOUT_ICON'] || '',
    FOOTER: process.env['VUE_APP_FOOTER'] || '',
  }),

  components: {
    UserAuthentication
  },

  created() {
    this.$root.$alert = (title, message = '', options = {}) => this.open(title, message, Object.assign({ alert: true }, options));
    this.$root.$confirm = (title, message = '', options = {}) => this.open(title, message, options);
    this.$root.$handleError = (e) => {
      console.log(e);
      this.open('An error occurred!', e?.response?.data?.error || e?.message || '', { alert: true, type: 'error' });
    };
    this.$root.$prompt = async(title, input = '', options = {}) => {
      const r = await this.open(title, input, Object.assign({ prompt: true }, options));
      return r ? this.message : r;
    };
  },

  methods: {
    async open(title, message, options) {
      if (this.promise) {
        this.dialog = false;
        await new Promise(resolve => setTimeout(resolve, 10));
        this.dialog = true;
        await this.promise;
      }

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

      this.promise = new Promise((resolve, reject) => {
        this.resolve = resolve;
        this.reject = reject;
      });

      return this.promise;
    },

    agree() {
      this.resolve(true);
      this.dialog = false;
      this.promise = null;
      this.resolve = null;
    },

    cancel() {
      this.resolve(false);
      this.dialog = false;
      this.promise = null;
      this.resolve = null;
    },

    clearSessions() {
      Object.keys(localStorage)
        .filter(e => e.startsWith('participant:') || e.startsWith('organizer:'))
        .forEach(e => localStorage.removeItem(e));
      localStorage.removeItem('credential');
      localStorage.removeItem('keys');
      location.reload();
    }
  },

  computed: {
    role() {
      return this.$store.state.user?.role;
    }
  },

  watch: {
    about(v) {
      localStorage.setItem('about-dismissed', String(!v));
    }
  }
}
</script>

<style>
#app {
  background-color: #e5e3dd;
}
</style>
