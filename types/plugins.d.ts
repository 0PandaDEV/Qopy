import { HistoryItem, ContentType } from './types';

declare module '#app' {
  interface NuxtApp {
    $history: ReturnType<typeof import('../plugins/history')>['provide']['history'];
    $settings: ReturnType<typeof import('../plugins/settings')>['provide']['settings'];
  }
}