<template>
  <v-dialog v-model="dialog" persistent max-width="400" scrollable>
    <template v-slot:activator="{ on, attrs }">
        <v-btn color="info" small v-bind="attrs" v-on="on">
          <v-icon left>mdi-chart-box-plus-outline</v-icon>
          Add Attribute Constraint
        </v-btn>
    </template>

    <v-card>
      <v-card-title class="text-h5 grey lighten-2">
        <v-icon left>mdi-chart-box-plus-outline</v-icon>

        Add Attribute Constraint

        <v-spacer />

        <v-btn icon @click="close()">
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </v-card-title>

      <v-form ref="form" lazy-validation @submit.prevent="add" v-if="attributes">
        <v-card-text>
          <v-select outlined label="Attribut" :items="attributes" :item-text="e => e[0]" return-object v-model="attribute" clearable />

          <v-text-field label="Lower Bound" v-if="attribute && attribute[1] === 'number'" v-model="lowerBound" prepend-inner-icon="mdi-greater-than-or-equal" :rules="[
            v => attribute[2] === undefined || Number(v) >= attribute[2] || `value has to be at least ${attribute[2]}`,
            v => attribute[3] === undefined || Number(v) <= attribute[3] || `value has to be at most ${attribute[3]}`
          ]" />

          <v-text-field label="Upper Bound" v-if="attribute && attribute[1] === 'number'" v-model="upperBound" prepend-inner-icon="mdi-less-than-or-equal" :rules="[
            v => attribute[2] === undefined || Number(v) >= attribute[2] || `value has to be at least ${attribute[2]}`,
            v => attribute[3] === undefined || Number(v) <= attribute[3] || `value has to be at most ${attribute[3]}`,
            v => lowerBound < v || `lower bound cannot be greater than upper bound`
          ]" />

          <v-select label="Valid Values" v-if="attribute && attribute[1] === 'select'" outlined chips multiple :items="attribute[2].map((e, i) => ({ text: e, value: i}))" :rules="[
            v => v && v.length > 0 || 'select at least one valid option`'
          ]" v-model="superset" hide-details>
            <template #prepend-inner>
              <span class="text-h6 pl-1 pr-2">∈</span>
            </template>
          </v-select>
        </v-card-text>

        <v-card-actions>
          <v-spacer />

          <v-btn type="submit" color="primary" :disabled="!attribute || !$refs.form">
            <v-icon small left>mdi-chart-box-plus-outline</v-icon>
            Add Constraint
          </v-btn>
        </v-card-actions>
      </v-form>
    </v-card>
  </v-dialog>
</template>

<script>
export default {
  name: 'AttributeConstraint',

  props: {
    attributes: Array
  },

  data() {
    return {
      dialog: false,
      attribute: null,
      lowerBound: null,
      upperBound: null,
      superset: null,
    };
  },

  methods: {
    async add() {
      if (!this.$refs.form?.validate?.()) {
        return;
      }

      const res = [ this.attributes.indexOf(this.attribute), this.attribute[1] ];

      switch (this.attribute[1]) {
        case 'number':
          res.push([Number(this.lowerBound), Number(this.upperBound)]);
          break;

        case 'select':
          res.push(this.superset);
          break;
      }

      this.$emit('add', res);
      this.dialog = false;
    },

    close() {
      this.dialog = false;
    }
  },

  watch: {
    dialog(v) {
      if (v) {
        this.attribute = null;
      }
    }
  }
}
</script>
