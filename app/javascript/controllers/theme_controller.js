import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static values = { fonts: Object }

  connect() {
    // Sync server-rendered theme to localStorage
    const serverTheme = this.element.dataset.theme
    if (serverTheme) {
      localStorage.setItem("theme", serverTheme)
    }
  }

  switch(event) {
    const theme = event.target.value || event.params.theme
    if (!theme) return

    // Apply theme to DOM immediately
    document.documentElement.dataset.theme = theme
    localStorage.setItem("theme", theme)

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
        jazz: "#0d0f1a"
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
      }).catch(() => {
        // Silent fail — localStorage already has it
      })
    }
  }
}
