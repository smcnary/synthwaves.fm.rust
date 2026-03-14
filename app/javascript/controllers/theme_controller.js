import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static values = { fonts: Object }

  connect() {
    this.syncOnRender = this.#syncThemeFromResponse.bind(this)
    document.addEventListener("turbo:before-render", this.syncOnRender)
  }

  disconnect() {
    document.removeEventListener("turbo:before-render", this.syncOnRender)
  }

  switch(event) {
    const theme = event.target.value || event.params.theme
    if (!theme) return

    // Apply theme to DOM immediately
    document.documentElement.dataset.theme = theme

    // Swap font stylesheet
    const fontLink = document.getElementById("theme-font")
    if (fontLink && this.fontsValue[theme]) {
      fontLink.href = this.fontsValue[theme]
    }

    // Update theme-color meta tag
    const metaThemeColor = document.querySelector('meta[name="theme-color"]')
    if (metaThemeColor) {
      const metaColors = {
        synthwave: "#0a0a1a",
        reggae: "#0d1a0d",
        punk: "#0a0a0a",
        jazz: "#0d0f1a",
        light: "#f8fafc"
      }
      metaThemeColor.content = metaColors[theme] || metaColors.synthwave
    }

    // Mark the selected radio card
    this.element.querySelectorAll("[data-theme-card]").forEach(card => {
      card.classList.toggle("ring-2", card.dataset.themeCard === theme)
      card.classList.toggle("ring-neon-cyan", card.dataset.themeCard === theme)
    })

    // Persist to server for logged-in users
    const csrfToken = document.querySelector('meta[name="csrf-token"]')?.content
    if (csrfToken) {
      fetch("/profile", {
        method: "PATCH",
        headers: {
          "Content-Type": "application/json",
          "X-CSRF-Token": csrfToken
        },
        body: JSON.stringify({ user: { theme: theme } })
      }).catch(() => {})
    }
  }

  // Sync data-theme from the server-rendered response on Turbo navigations
  #syncThemeFromResponse(event) {
    const newTheme = event.detail.newBody.parentElement?.dataset?.theme
    if (newTheme) {
      document.documentElement.dataset.theme = newTheme
    }
  }
}
