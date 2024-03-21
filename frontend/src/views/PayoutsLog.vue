<template>
  <div>
    <h1 class="my-3">Payouts Log</h1>

    <v-alert type="info" text>
      This view only exists for demonstration purposes and shows all payouts issued by the service, which would normally only be accessible to an institution's accounting department.
    </v-alert>

    <v-data-table dense :headers="headers" :items="payouts" class="elevation-1">
      <template v-slot:[`item.receipt`]="{ item }">
        <v-btn small icon @click="showReceipt(item)">
          <v-icon>mdi-receipt</v-icon>
        </v-btn>
      </template>
    </v-data-table>
  </div>
</template>

<script>
import axios from 'axios';

export default {
  name: 'PayoutsLog',

  data: () => ({
    payouts: [],
    headers: [{
      text: '#',
      align: 'start',
      value: 'id'
    }, {
      text: 'Recipient',
      align: 'start',
      value: 'recipient'
    }, {
      text: 'Value',
      value: 'value',
      align: 'end'
    }, {
      text: 'Target',
      value: 'target',
      align: 'start'
    }, {
      text: 'Receipt',
      value: 'receipt',
      align: 'end'
    }]
  }),

  mounted() {
    this.reload();
  },

  methods: {
    async reload() {
      const res = await axios.get(`/api/demo/payouts`);
      this.payouts = res.data;
    },

    async showReceipt(item) {
      const receipt = item.receipt.match(/.{1,43}/g).join('\n');
      await this.$root.$alert(`Receipt #${item.id}`, receipt, {
        width: '600px',
        type: 'info',
        style: {
          fontFamily: 'monospace',
          fontSize: 'larger',
          textAlign: 'center'
        }
      });
    }
  },
}
</script>
