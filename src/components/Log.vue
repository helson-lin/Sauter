<template>
  <div class="log w-full h-full overflow-y-auto" ref="logRef" id="logDom">
    <p v-for="item in htmlString" :key="item.text" :class="[`log-${item.type}`, 'log-item']">{{ item.type }} {{ item.time }}: {{ item.text }}</p>
  </div>
</template>
<script>
import { defineComponent, computed, ref, } from 'vue';
export default defineComponent({
  props: {
    text: {
      type: Array,
      required: true,
    },
  },
  setup(props) {
      const logRef = ref(null);
      const htmlString = computed(() => {
          // 创建 props.text 的副本并反转
          return props.text ? [...props.text].reverse() : null;
      });  
      return {
        htmlString,
        logRef
      };
  }
});
</script>

<style lang="scss" scoped>

#logDom {
  color: var(--primary-color);
}

.log-success {
  color: #18A058;
}

.log-error {
  color: rgba(192, 26, 22, 1);
}

.log-warning {
  color: rgba(255, 204, 2, 1);
}

.log-title {
  font-weight: bold;
}

.log-item {
  font-weight: 500;
}
</style>
