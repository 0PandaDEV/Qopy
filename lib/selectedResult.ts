import type { HistoryItem } from '~/types/types'

interface GroupedHistory {
  label: string
  items: HistoryItem[]
}

export const selectedGroupIndex = ref(0)
export const selectedItemIndex = ref(0)
export const selectedElement = ref<HTMLElement | null>(null)

export const useSelectedResult = (groupedHistory: Ref<GroupedHistory[]>) => {
  const selectedItem = computed<HistoryItem | null>(() => {
    const group = groupedHistory.value[selectedGroupIndex.value]
    return group?.items[selectedItemIndex.value] ?? null
  })

  const isSelected = (groupIndex: number, itemIndex: number): boolean => {
    return selectedGroupIndex.value === groupIndex && selectedItemIndex.value === itemIndex
  }

  const selectNext = (): void => {
    const currentGroup = groupedHistory.value[selectedGroupIndex.value]
    if (selectedItemIndex.value < currentGroup.items.length - 1) {
      selectedItemIndex.value++
    } else if (selectedGroupIndex.value < groupedHistory.value.length - 1) {
      selectedGroupIndex.value++
      selectedItemIndex.value = 0
    }
  }

  const selectPrevious = (): void => {
    if (selectedItemIndex.value > 0) {
      selectedItemIndex.value--
    } else if (selectedGroupIndex.value > 0) {
      selectedGroupIndex.value--
      selectedItemIndex.value = groupedHistory.value[selectedGroupIndex.value].items.length - 1
    }
  }

  const selectItem = (groupIndex: number, itemIndex: number): void => {
    selectedGroupIndex.value = groupIndex
    selectedItemIndex.value = itemIndex
  }

  return {
    selectedItem,
    isSelected,
    selectNext,
    selectPrevious,
    selectItem,
    selectedElement
  }
} 