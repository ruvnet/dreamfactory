import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { Theme } from '@/types'

interface Toast {
  id: string
  type: 'success' | 'error' | 'warning' | 'info'
  title: string
  message?: string
  duration?: number
}

interface Modal {
  id: string
  component: React.ComponentType<any>
  props?: Record<string, any>
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full'
}

interface UIState {
  theme: Theme
  sidebarOpen: boolean
  sidebarCollapsed: boolean
  toasts: Toast[]
  modals: Modal[]
  isLoading: boolean
  loadingMessage?: string
}

interface UIActions {
  setTheme: (theme: Theme) => void
  toggleSidebar: () => void
  setSidebarOpen: (open: boolean) => void
  toggleSidebarCollapsed: () => void
  setSidebarCollapsed: (collapsed: boolean) => void
  addToast: (toast: Omit<Toast, 'id'>) => void
  removeToast: (id: string) => void
  clearToasts: () => void
  openModal: (modal: Omit<Modal, 'id'>) => void
  closeModal: (id: string) => void
  closeAllModals: () => void
  setLoading: (loading: boolean, message?: string) => void
}

type UIStore = UIState & UIActions

export const useUIStore = create<UIStore>()(
  persist(
    (set, get) => ({
      // Initial state
      theme: 'system',
      sidebarOpen: true,
      sidebarCollapsed: false,
      toasts: [],
      modals: [],
      isLoading: false,
      loadingMessage: undefined,

      // Actions
      setTheme: (theme: Theme) => {
        set({ theme })
      },

      toggleSidebar: () => {
        set((state) => ({ sidebarOpen: !state.sidebarOpen }))
      },

      setSidebarOpen: (sidebarOpen: boolean) => {
        set({ sidebarOpen })
      },

      toggleSidebarCollapsed: () => {
        set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed }))
      },

      setSidebarCollapsed: (sidebarCollapsed: boolean) => {
        set({ sidebarCollapsed })
      },

      addToast: (toastData: Omit<Toast, 'id'>) => {
        const toast: Toast = {
          id: Math.random().toString(36).substring(2, 9),
          duration: 5000,
          ...toastData,
        }
        set((state) => ({ toasts: [...state.toasts, toast] }))

        // Auto-remove toast after duration
        if (toast.duration && toast.duration > 0) {
          setTimeout(() => {
            get().removeToast(toast.id)
          }, toast.duration)
        }
      },

      removeToast: (id: string) => {
        set((state) => ({
          toasts: state.toasts.filter((toast) => toast.id !== id),
        }))
      },

      clearToasts: () => {
        set({ toasts: [] })
      },

      openModal: (modalData: Omit<Modal, 'id'>) => {
        const modal: Modal = {
          id: Math.random().toString(36).substring(2, 9),
          size: 'md',
          ...modalData,
        }
        set((state) => ({ modals: [...state.modals, modal] }))
      },

      closeModal: (id: string) => {
        set((state) => ({
          modals: state.modals.filter((modal) => modal.id !== id),
        }))
      },

      closeAllModals: () => {
        set({ modals: [] })
      },

      setLoading: (isLoading: boolean, loadingMessage?: string) => {
        set({ isLoading, loadingMessage })
      },
    }),
    {
      name: 'ui-storage',
      partialize: (state) => ({
        theme: state.theme,
        sidebarCollapsed: state.sidebarCollapsed,
      }),
    }
  )
)