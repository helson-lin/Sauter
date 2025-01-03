import globals from 'globals';
import pluginJs from '@eslint/js';
import pluginVue from 'eslint-plugin-vue';


export default [
  {
    files: ['**/*.{js,mjs,cjs,vue}'],
  },
  { languageOptions: { globals: { ...globals.browser, ...globals.node } } },
  pluginJs.configs.recommended,
  ...pluginVue.configs['flat/essential'],
  {
    rules: {
      // Example rules
      'no-console': 'warn', // Warn on console statements
      'no-debugger': 'error', // Error on debugger statements
      'vue/valid-v-model': 'off',
      'vue/multi-word-component-names': 'off', // Disable multi-word component name rule
      'semi': ['error', 'always'], // Require semicolons
      'quotes': ['error', 'single'], // Require single quotes
    }
  }
];