import { Key, keyboard } from "wrdu-keyboard";
import { platform } from "@tauri-apps/plugin-os";

type KeyboardHandler = (event: KeyboardEvent) => void;

const activeContexts = new Set<string>();
const handlersByContext: Record<
  string,
  Array<{
    keys: Key[];
    callback: KeyboardHandler;
    prevent: boolean;
    priority?: number;
  }>
> = {};

const PRIORITY = {
  HIGH: 100,
  MEDIUM: 50,
  LOW: 0,
};

let currentOS = "windows";
const initOS = async () => {
  currentOS = await platform();
};

const useKeyboard = {
  PRIORITY,

  registerContext: (contextName: string) => {
    if (!handlersByContext[contextName]) {
      handlersByContext[contextName] = [];
    }
  },

  enableContext: (contextName: string) => {
    if (!handlersByContext[contextName]) {
      useKeyboard.registerContext(contextName);
    }
    activeContexts.add(contextName);

    initKeyboardHandlers();
  },

  disableContext: (contextName: string) => {
    activeContexts.delete(contextName);

    initKeyboardHandlers();
  },

  on: (
    contextName: string,
    keys: Key[],
    callback: KeyboardHandler,
    options: { prevent?: boolean; priority?: number } = {}
  ) => {
    if (!handlersByContext[contextName]) {
      useKeyboard.registerContext(contextName);
    }
    handlersByContext[contextName].push({
      keys,
      callback,
      prevent: options.prevent ?? true,
      priority: options.priority ?? PRIORITY.LOW,
    });

    if (activeContexts.has(contextName)) {
      initKeyboardHandlers();
    }
  },

  clearAll: () => {
    keyboard.clear();
  },

  setupAppShortcuts: (options: {
    onNavigateUp?: () => void;
    onNavigateDown?: () => void;
    onSelect?: () => void;
    onEscape?: () => void;
    onToggleActions?: () => void;
    contextName?: string;
    priority?: number;
  }) => {
    const {
      onNavigateUp,
      onNavigateDown,
      onSelect,
      onEscape,
      onToggleActions,
      contextName = "app",
      priority = PRIORITY.LOW,
    } = options;

    if (!handlersByContext[contextName]) {
      useKeyboard.registerContext(contextName);
    }

    if (onNavigateUp) {
      useKeyboard.on(contextName, [Key.UpArrow], () => onNavigateUp(), {
        priority,
      });
    }

    if (onNavigateDown) {
      useKeyboard.on(contextName, [Key.DownArrow], () => onNavigateDown(), {
        priority,
      });
    }

    if (onSelect) {
      useKeyboard.on(contextName, [Key.Enter], () => onSelect(), { priority });
    }

    if (onEscape) {
      useKeyboard.on(contextName, [Key.Escape], () => onEscape(), { priority });
    }

    if (onToggleActions) {
      const togglePriority = PRIORITY.HIGH;
      
      if (currentOS === "macos") {
        useKeyboard.on(
          contextName,
          [Key.LeftMeta, Key.K],
          () => onToggleActions(),
          { priority: togglePriority }
        );
        useKeyboard.on(
          contextName,
          [Key.RightMeta, Key.K],
          () => onToggleActions(),
          { priority: togglePriority }
        );
      } else {
        useKeyboard.on(
          contextName,
          [Key.LeftControl, Key.K],
          () => onToggleActions(),
          { priority: togglePriority }
        );
        useKeyboard.on(
          contextName,
          [Key.RightControl, Key.K],
          () => onToggleActions(),
          { priority: togglePriority }
        );
      }
    }
  },

  setupKeybindCapture: (options: {
    onCapture: (key: string) => void;
    onComplete: () => void;
  }) => {
    const { onCapture, onComplete } = options;

    keyboard.prevent.down([Key.All], (event: KeyboardEvent) => {
      if (event.code === "Escape") {
        onComplete();
        return;
      }
      onCapture(event.code);
    });
  },
};

const initKeyboardHandlers = () => {
  keyboard.clear();

  let allHandlers: Array<{ keys: Key[], callback: KeyboardHandler, prevent: boolean, priority: number, contextName: string }> = [];
  
  for (const contextName of activeContexts) {
    const handlers = handlersByContext[contextName] || [];
    allHandlers = [...allHandlers, ...handlers.map(handler => ({
      ...handler,
      priority: handler.priority ?? PRIORITY.LOW,
      contextName
    }))];
  }
  
  allHandlers.sort((a, b) => b.priority - a.priority);
  
  const handlersByKeyCombination: Record<string, Array<typeof allHandlers[0]>> = {};
  
  allHandlers.forEach(handler => {
    const keyCombo = handler.keys.join('+');
    if (!handlersByKeyCombination[keyCombo]) {
      handlersByKeyCombination[keyCombo] = [];
    }
    handlersByKeyCombination[keyCombo].push(handler);
  });
  
  Object.values(handlersByKeyCombination).forEach(handlers => {
    const handler = handlers[0];
    
    const wrappedCallback: KeyboardHandler = (event) => {
      const isMetaCombo = handler.keys.length > 1 && 
        (handler.keys.includes(Key.LeftMeta) || 
         handler.keys.includes(Key.RightMeta) || 
         handler.keys.includes(Key.LeftControl) ||
         handler.keys.includes(Key.RightControl));
      
      const isNavigationKey = event.key === 'ArrowUp' || 
                             event.key === 'ArrowDown' || 
                             event.key === 'Enter' || 
                             event.key === 'Escape';
      
      const isInInput = event.target instanceof HTMLInputElement || 
                        event.target instanceof HTMLTextAreaElement;
      
      if (isMetaCombo || isNavigationKey || !isInInput) {
        handler.callback(event);
      }
    };
    
    if (handler.prevent) {
      keyboard.prevent.down(handler.keys, wrappedCallback);
    } else {
      keyboard.down(handler.keys, wrappedCallback);
    }
  });
};

export default defineNuxtPlugin(async () => {
  await initOS();

  initKeyboardHandlers();

  return {
    provide: {
      keyboard: {
        ...useKeyboard,
        Key,
      },
    },
  };
});
