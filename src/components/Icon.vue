<template>
  <n-icon :size="iconSize">
    <svg aria-hidden="true">
      <use :xlink:href="symbolId" />
    </svg>
  </n-icon>
</template>

<script>
import { computed, defineComponent } from 'vue';
export default defineComponent({
  props: {
    name: {
    type: String,
    required: true,
  },
  size: {
    type: Number,
    default: 15,
  },
  },
  setup(props) {
    const symbolId = computed(() => `#icon-svg-${props.name}`);
    const iconSize = computed(() => props.size);
    return {
      iconSize,
      symbolId
    };
  }
});

</script>

<style scoped>
.n-icon {
  :deep(path) {
    fill: red;
    stroke: red;
  }
}
</style>
