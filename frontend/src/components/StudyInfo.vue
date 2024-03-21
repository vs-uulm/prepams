<template>
  <div>
    <v-chip color="primary" outlined pill small class="mr-2">
      <v-icon small left>
        mdi-database
      </v-icon>
      {{ study.reward }} Credit{{ study.reward > 1 ? 's' : '' }}
    </v-chip>

    <v-chip color="secondary" outlined pill small class="mr-2">
      <v-icon small left>
        mdi-timer-sand
      </v-icon>
      {{ study.duration }}
    </v-chip>

    <v-chip color="success" outlined pill small class="mr-2" v-if="study.participations">
      <v-icon small left>
        mdi-sticker-check-outline
      </v-icon>
      {{ study.participations.length }}
      completed participation{{ study.participations.length !== 1 ? 's' : '' }}
    </v-chip>
  </div>
</template>

<script>
export default {
  name: 'StudyInfo',

  props: {
    study: Object
  }
}
</script>
