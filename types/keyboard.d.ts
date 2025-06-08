import type { Key as WaraduKey, useKeyboard } from '@waradu/keyboard';

declare module '#app' {
  interface NuxtApp {
    $keyboard: {
      listen: ReturnType<typeof useKeyboard>['listen'];
      init: ReturnType<typeof useKeyboard>['init'];
      Key: typeof WaraduKey;
      currentOS: string;
      clearAll: () => void;
    };
  }
}

declare module 'vue' {
  interface ComponentCustomProperties {
    $keyboard: {
      listen: ReturnType<typeof useKeyboard>['listen'];
      init: ReturnType<typeof useKeyboard>['init'];
      Key: typeof WaraduKey;
      currentOS: string;
      clearAll: () => void;
    };
  }
}

export {};