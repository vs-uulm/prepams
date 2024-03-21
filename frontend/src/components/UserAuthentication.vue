<template>
  <div>
    <v-select outlined prepend-inner-icon="mdi-account-switch-outline" item-text="id" item-value="id"
      :items="$store.state.credentials" type="text" label="Currently used credential" :value="$store.state.credential"
      :menu-props="{ offsetY: true }" class="mt-4 mb-2" @input="switchAccount" messages="foo">
      <template #item="{ item, on, attrs }">
        <v-list-item v-on="on" v-bind="attrs">
          <v-list-item-icon>
            <v-icon>{{ item.role === 'none' ? 'mdi-cancel' : roles.find(e => e.value === item.role).icon }}</v-icon>
          </v-list-item-icon>
          <v-list-item-content>
            <v-list-item-title>{{ item.id }}</v-list-item-title>
          </v-list-item-content>
        </v-list-item>
      </template>
      <template #selection="{item}">
        <span :class="{ blurry: hide }">{{ item && item.id ? item.id : '' }}</span>
      </template>
      <template #message>
        <div class="d-flex">
          <div>
            <v-icon color="info" small left>mdi-information-outline</v-icon>
          </div>
          <div class="info--text">
            For demonstration purposes this version of the PrePaMS client can simultaneously store multiple credentials
            and supports easy switching between accounts.
          </div>
        </div>
      </template>
    </v-select>

    <template v-if="!$store.state.user">
      <v-card class="mb-4">
        <v-card-title>
          Log Into Existing Account
        </v-card-title>

        <v-card-subtitle>
          You have an account and want to sign in again?
        </v-card-subtitle>

        <v-card-text>
          You switched your device or cleared your browser data?
          Click the "sign in with password" button to log into your existing account.
        </v-card-text>

        <v-card-actions>
          <v-spacer />

          <v-btn @click="signIn()">
            <v-icon left>mdi-clipboard-edit-outline</v-icon>
            sign in with password
          </v-btn>
        </v-card-actions>
      </v-card>

      <v-card class="mb-4">
        <v-card-title>
          Request New Account
        </v-card-title>

        <v-card-subtitle>
          You do not yet have an account?
        </v-card-subtitle>

        <v-card-text>
          Click the "request account" button to request a PrePaMS account.
          In a practical deployment this would take you to your instution's authentication service.
          For demonstration purposes this allows you to simply create any account you want.
        </v-card-text>

        <v-card-actions>
          <v-spacer />
          <v-btn @click="signUp()">
            <v-icon left>mdi-clipboard-edit-outline</v-icon>
            Request Account
          </v-btn>
        </v-card-actions>
      </v-card>

      <v-card>
        <v-card-title>
          Recover Existing Account
        </v-card-title>

        <v-card-subtitle>
          You forgot your password and want to access your account?
        </v-card-subtitle>

        <v-card-text>
          Click the "recover account" button to log into your existing account.
          You will then be prompted to scan your recovery code.
        </v-card-text>

        <v-card-actions>
          <v-spacer />

          <v-dialog v-model="scanDialog" max-width="500" persistent>
            <template v-slot:activator="{ on, attrs }">
              <v-btn v-bind="attrs" v-on="on">
                <v-icon left>mdi-data-matrix-scan</v-icon>
                Recover Account
              </v-btn>
            </template>

            <v-card @drop.prevent="onDrop($event)" @dragover.prevent @dragenter.prevent @dragleave.prevent>
              <v-card-title class="text-h5 grey lighten-2">
                Scan your recovery code

                <v-spacer />

                <v-btn icon @click="scanDialog = false">
                  <v-icon>mdi-close</v-icon>
                </v-btn>
              </v-card-title>

              <v-card-text class="py-6">
                <v-alert type="info" v-show="!loading">
                  Drag and drop your saved recovery code file here or scan the code with your camera.

                  <br><br>

                  <v-file-input placeholder="Choose File..." solo prepend-inner-icon="mdi-file-image-outline"
                    prepend-icon="" hide-details="" @change="onDrop($event)" />
                </v-alert>

                <v-progress-linear height="20" indeterminate striped v-if="loading" />
                <video ref="video" v-show="!loading" style="width: 100%; height: 100%; display: block; object-fit: cover;"
                  autoplay muted playsinline />
              </v-card-text>
            </v-card>
          </v-dialog>
        </v-card-actions>
      </v-card>
    </template>

    <v-card class="mb-4" v-if="$store.state.user">
      <v-card-title>
        <v-icon left>mdi-wallet</v-icon>
        <span :class="{blurry: hide}">{{ $store.state.user.id }}</span>

        <v-spacer />

        <v-progress-circular :indeterminate="loading" v-if="loading" />
        <v-icon @click="hide = !hide">{{ hide ? 'mdi-eye-outline' : 'mdi-eye-off-outline' }}</v-icon>
      </v-card-title>

      <v-card-text v-if="$store.state.user.role === 'participant'" :class="{ blurry: hide }">
        <v-chip class="ma-2 px-4" color="primary" outlined pill close-icon="mdi-refresh" close @click:close="refreshBalance()">
          <v-icon left>
            mdi-database
          </v-icon>

          Balance: {{ $store.state.user.balance }}
        </v-chip>
        <v-chip v-for="([attr, type, ...params], i) in attributes" :key="i" label class="ma-2 mt-n1 px-4 d-inline-block" outlined color="secondary">
          <v-icon left>
            mdi-chart-box-outline
          </v-icon>

          <b>{{attr}}:&nbsp;</b>
          {{ type === 'select' ? params[0][$store.state.user.attributes[i]] : $store.state.user.attributes[i] }}
        </v-chip>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn small class="pr-3" color="primary" @click="requestPayout()" :disabled="loading || !(this.$store.state.user.balance > 0)" :loading="loading">
          <v-icon left>mdi-database-export</v-icon>
          Request Payout
        </v-btn>
        <v-btn small v-if="this.$store.state.user" @click="$store.dispatch('lock')" class="pr-3" color="warning" :disabled="loading" :loading="loading">
          <v-icon left>mdi-shield-key-outline</v-icon>
          Lock Account
        </v-btn>
      </v-card-actions>
    </v-card>

    <v-dialog v-model="dialog" :max-width="recoveryCode ? 1000 : 500" persistent scrollable>
      <v-form @submit.prevent="submit()" :loading="loading" :disabled="loading">
        <v-card>
          <v-card-title class="text-h5 grey lighten-2">
            <v-icon left>{{ recoveryCode ? 'mdi-backup-restore' : mode === 'mdi-account-plus' ?
              'mdi-clipboard-edit-outline' : mode === 'signin' ? 'mdi-shield-key-outline' : 'mdi-lock-outline' }}
            </v-icon>
            {{ recoveryCode ? 'Backup your recovery code' : mode === 'signup' ? 'Request New Account' : mode ===
            'signin' ? 'Sign in with password' : 'Enter your password to unlock' }}

            <v-spacer />

            <v-btn icon @click="dialog = false">
              <v-icon>mdi-close</v-icon>
            </v-btn>
          </v-card-title>

          <v-card-text class="py-6" v-if="loading">
            <v-progress-linear height="20" indeterminate striped />
          </v-card-text>

          <v-card-text class="pb-n2" v-else-if="recoveryCode">
            <v-row>
              <v-col cols="12" md="5" class="pt-6">
                <v-alert type="info" dense>
                  This QR code is your recovery key.
                  If you loose access to this device, clear your browser data, or forget your password, you need this
                  code to get access to your account.
                  In case you loose this code and your device/browser data, your account can no longer be
                  accessed.<br><br>
                  Treat this code the same way as you would a password, because anyone can get access to your account
                  and your participations with this code!
                  You can press the download button below, to download this code that you can print out.
                  Keep a physical copy in your sock drawer to be on the safe side :)
                </v-alert>

                <div class="text-center">
                  <v-btn color="warning" :href="recoveryFile" download="PrePaMS-recovery-code.png"
                    @click="codeDownloaded = true">
                    <v-icon left>mdi-file-image-outline</v-icon>
                    Download Recovery Code
                  </v-btn>
                </div>
              </v-col>
              <v-col cols="12" md="7">
                <v-img :src="recoveryFile" contain />
              </v-col>
            </v-row>
          </v-card-text>

          <v-card-text class="pt-6 pb-0" v-else>
            <v-select outlined prepend-inner-icon="mdi-account-question" :items="roles" type="text" label="Account Type"
              v-model="role" :menu-props="{ offsetY: true }" :disabled="switching" v-if="mode !== 'unlock'">
              <template #item="{ item, on, attrs }">
                <v-list-item v-on="on" v-bind="attrs">
                  <v-list-item-icon>
                    <v-icon>{{ item.icon }}</v-icon>
                  </v-list-item-icon>
                  <v-list-item-content>
                    <v-list-item-title>{{ item.text }}</v-list-item-title>
                  </v-list-item-content>
                </v-list-item>
              </template>
            </v-select>

            <v-text-field outlined prepend-inner-icon="mdi-at" type="text" label="Email Address" v-model="id"
              autocomplete="username" :disabled="switching" v-if="mode !== 'unlock'" />

            <v-text-field outlined prepend-inner-icon="mdi-shield-key-outline" type="password" label="Password"
              v-model="password" :error-messages="errors" autofocus :autocomplete="mode === 'signup' || mode === 'signin' ? 'new-password' : 'password'" />

            <v-alert outlined type="info" dense v-if="mode === 'signup'">
              When you don't use PrePaMS for a while, we will lock your account to prevent misuse.
              Specify the password that you want to use to unlock your account.
              PrePaMS will never send your password to a server, it will never leave your device under any circumstance.
            </v-alert>

            <v-alert outlined type="info" dense v-if="mode === 'unlock'">
              When you don't use PrePaMS for a while, we will lock your account to prevent misuse.
              Enter your password to unlock your account.
            </v-alert>

            <v-card outlined v-if="attributes && (mode === 'signup' || mode === 'signin') && role === 'participant'">
              <v-card-text>
                <div class="text-overline mb-2 mt-n2">
                  ATTRIBUTES
                </div>

                <template v-for="([attr, type, ...params], i) in attributes">
                  <v-text-field v-if="type === 'number'" :key="`n${i}`" outlined dense type="number" :label="attr" :rules="[
                    v => params[0] === undefined || Number(v) >= params[0] || `value has to be at least ${params[0]}`,
                    v => params[1] === undefined || Number(v) <= params[1] || `value has to be at most ${params[1]}`
                  ]" v-model="values[i]" :autocomplete="false" class="mb-2" />

                  <v-select v-if="type === 'select'" :key="`s${i}`" outlined dense :label="attr" :items="params[0].map((e, i) => ({ text: e, value: i}))" v-model="values[i]" />
                </template>

                <v-alert outlined type="info" dense class="mb-n1">
                  These attributes will help researchers select an appropriate sample for their studies.
                  They are provided to the credential issuer during registration so that they can verify the values you provide.
                  In return, you will receive a signed certificate which allows you to prove the above values anonymously without disclosing the values and your identity to researchers.
                </v-alert>
              </v-card-text>
            </v-card>
          </v-card-text>

          <v-divider />

          <v-card-actions>
            <v-spacer />
            <v-btn type="submit" @click="submit()" color="primary" text :loading="loading" :disabled="loading">
              {{ recoveryCode ? 'Finish' : mode === 'signup' ? 'Request Account' : mode === 'signin' ? 'sign in with password' : 'Unlock' }}
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-form>
    </v-dialog>

    <v-dialog v-model="working" persistent width="300">
      <v-card>
        <v-card-text class="pt-3">
          <b>Processing...</b>
          <v-progress-linear indeterminate class="mt-2" />
      </v-card-text>
      </v-card>
    </v-dialog>
  </div>
</template>

<script>
import jsQR from 'jsqr';
import axios from 'axios';

export default {
  name: 'UserAuthentication',

  data: () => ({
    hide: false,
    scanning: false,
    scanDialog: false,
    scanCanvas: null,
    attributes: null,

    dialog: false,
    switching: false,
    loading: false,
    codeDownloaded: false,
    recoveryCode: null,
    recoveryFile: null,

    working: false,

    roles: [{
      text: 'Participant',
      value: 'participant',
      icon: 'mdi-account'
    }, {
      text: 'Organizer',
      value: 'organizer',
      icon: 'mdi-account-tie'
    }],

    mode: 'signup',
    role: 'participant',
    id: '',
    password: '',
    values: [],
    errors: []
  }),

  async mounted() {
    const res = await axios.get('/api/issuer/attributes');
    this.attributes = res.data;

    if (localStorage.getItem('credential')) {
      setTimeout(() => this.switchAccount(localStorage.getItem('credential')), 250);
    } else {
      this.id = 'mail@example.com';
      this.password = 'example';
      this.values = ['1993', 1];
      this.signUp();
    }
  },

  methods: {
    async signUp() {
      this.switching = false;
      this.dialog = 'true';
      this.mode = 'signup';
    },

    async signIn() {
      this.switching = false;
      this.dialog = 'true';
      this.mode = 'signin';
    },

    async submit() {
      if (this.loading) {
        return;
      }

      if (this.recoveryCode) {
        if (!this.codeDownloaded) {
          if (await this.$root.$confirm('Are you sure?', 'You should download and backup your recovery code, before continuing!', {
            cancelText: 'Continue without Backup',
            confirmText: 'Okay'
          })) {
            return;
          }
        }

        this.dialog = false;
        this.recoveryCode = false;
        this.recoveryFile = false;
        return;
      }

      try {
        this.loading = true;

        const res = await this.$store.dispatch(this.mode === 'signup' ? this.mode : 'signin', {
          id: this.id,
          role: this.role,
          password: this.password,
          attributes: this.values
        });
        this.password = '';

        if (res?.recoveryFile && res?.recoveryCode) {
          this.recoveryFile = res.recoveryFile;
          this.recoveryCode = res.recoveryCode;
        } else {
          this.dialog = false;
        }
      } catch (e) {
        console.log(e);
        if (e?.request?.response) {
          this.errors = [JSON.parse(e?.request?.response)?.error];
        } else {
          this.errors = [typeof e === 'string' ? e : e.message];
        }
      } finally {
        this.loading = false;
      }
    },

    async onDrop(e) {
      if (!e) {
        return;
      }

      const files = e instanceof File ? [e] : (e.target.files || e.dataTransfer.files);
      if (!files.length) {
        return;
      }

      this.loading = true;
      this.scanning = false;

      try {
        const file = await new Promise((resolve, reject) => {
          const fileReader = new FileReader();
          fileReader.onload = () => resolve(fileReader.result);
          fileReader.onerror = (e) => reject(e);

          if (files[0].type.startsWith('image')) {
            fileReader.readAsDataURL(files[0]);
          } else {
            reject(new Error('invalid format'));
          }
        });

        const canvas = document.createElement('canvas');
        const context = canvas.getContext('2d');

        if (files[0].type.startsWith('image')) {
          const img = document.createElement('img');
          img.src = file;
          await new Promise((resolve) => img.onload = () => resolve());

          canvas.width = img.width;
          canvas.height = img.height;
          context.drawImage(img, 0, 0);
        } else {
          throw new Error('Only image formats are supported!');
        }

        const imageData = context.getImageData(0, 0, canvas.width, canvas.height);
        const code = jsQR(imageData.data, imageData.width, imageData.height);
        if (!code?.binaryData) {
          throw new Error('Could not detect a recovery code.');
        }

        await this.$store.dispatch('recover', code.binaryData);
        this.scanDialog = false;
        this.loading = false;
        this.dialog = false;
        this.scanning = false;
      } catch (e) {
        console.log(e);
        this.loading = false;
        this.scanning = false;
        await this.$root.$alert(`Error: Failed to Recover Account`, e.message, { type: 'error' });
      }
    },

    async scanTick() {
      if (this.$refs.video && this.$refs.video.readyState === this.$refs.video.HAVE_ENOUGH_DATA) {
        this.loading = false;

        if (this.scanning) {
          this.scanCanvas.width = this.$refs.video.videoWidth;
          this.scanCanvas.height = this.$refs.video.videoHeight;
          this.scanContext.drawImage(this.$refs.video, 0, 0, this.scanCanvas.width, this.scanCanvas.height);
          const imageData = this.scanContext.getImageData(0, 0, this.scanCanvas.width, this.scanCanvas.height);
          const code = jsQR(imageData.data, imageData.width, imageData.height);

          if (code?.binaryData) {
            try {
              await this.$store.dispatch('recover', code.binaryData);
              this.scanDialog = false;
              this.loading = false;
              this.dialog = false;
              this.scanning = false;
            } catch (e) {
              await this.$root.$alert(`Error: Failed to Recover Account`, e.message, { type: 'error' });
            }
          }
        }
      }
      setImmediate(this.scanTick);
    },

    async switchAccount(id) {
      try {
        await this.$store.dispatch('switchAccount', id);
      } catch (e) {
        console.log(e);
        if (e.role) {
          this.id = id;
          this.dialog = 'true';
          this.mode = 'signin';
          this.role = e.role || 'participant';
          this.switching = true;
        }
      }
    },

    async refreshBalance() {
      this.working = true;
      setTimeout(async () => {
        try {
          await this.$store.dispatch('refreshBalance');
        } catch (e) {
          this.$root.$handleError(e);
        } finally {
          this.working = false;
        }
      }, 50);
    },

    async requestPayout() {
      if (!(this.$store.state.user.balance > 0)) {
        return;
      }

      const request = {
        source: this,
        id: this.$store.state.user.id
      };

      const balance = this.$store.state.user.balance;
      let valid = false;
      while (!valid) {
        try {
          const input = await this.$root.$prompt('How many credits you want to get paid out?', '', { hint: `Enter a whole number between 0 and ${balance}` });
          if (input === false) {
            return;
          }

          const amount = parseInt(input, 10);
          if (amount >= 0 && amount <= balance) {
            request.amount = amount;
            valid = true;
            break;
          }
        } catch {
          // ignore
        }
      }

      if (!request.amount === 0) {
        return;
      }

      const target = await this.$root.$confirm('Select your payout target', '', {
        confirmText: 'ECTS',
        cancelText: 'Money'
      });

      request.target = target ? 'ECTS' : 'Money';

      this.working = true;
      setTimeout(async () => {
        try {
          const res = await this.$store.dispatch('payout', request);
          await this.$store.dispatch('refreshBalance');
          if (res?.receipt) {
            const receipt = res.receipt.match(/.{1,43}/g).join('\n');
            await this.$root.$alert('Payout Receipt', receipt, {
              width: '600px',
              type: 'info',
              style: {
                fontFamily: 'monospace',
                fontSize: 'larger',
                textAlign: 'center'
              }
            });
          }
        } catch (e) {
          this.$root.$handleError(e);
        } finally {
          this.working = false;
        }
      }, 50);
    }
  },

  watch: {
    '$store.state.user': {
      handler(v) {
        if (v === null && this.$store.state.credential !== 'Unauthenticated') {
          this.mode = 'unlock';
          this.dialog = true;
        } else if (v && v.role === 'participant') {
          this.$store.dispatch('refreshBalance');
        }
      }
    },

    async scanDialog(v) {
      if (v) {
        this.scanCanvas = document.createElement('canvas');
        this.scanContext = this.scanCanvas.getContext('2d');

        const devices = await navigator.mediaDevices.enumerateDevices();
        const videoDevices = devices.filter(d => d.kind === 'videoinput').reverse();

        for (const device of videoDevices) {
          try {
            const stream = await navigator.mediaDevices.getUserMedia({
              audio: false,
              video: {
                facingMode: { ideal: 'environment' },
                width: { min: 360, ideal: 640, max: 1920 },
                height: { min: 240, ideal: 480, max: 1080 },
              },
              deviceId: device.deviceId ? { exact: device.deviceId } : undefined
            });

            this.$refs.video.srcObject = stream;
            this.$refs.video.setAttribute('playsinline', true);
            this.$refs.video.play();
            this.scanning = true;
            requestAnimationFrame(this.scanTick);
            break;
          } catch {
            // ignore
          }
        }
      } else {
        if (this.$refs.video?.srcObject) {
          this.$refs.video.srcObject.getTracks()[0].stop();
        }
      }
    }
  }
}
</script>

<style>
.blurry {
  filter: blur(8px);
}
</style>
