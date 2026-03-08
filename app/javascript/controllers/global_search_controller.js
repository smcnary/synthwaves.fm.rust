import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["input", "dropdown", "frame"]
  static values = { url: String }

  connect() {
    this.timeout = null
    this.handleClickOutside = this.handleClickOutside.bind(this)
    document.addEventListener("click", this.handleClickOutside)
    this.element.addEventListener("turbo:frame-load", () => this.showDropdown())
  }

  disconnect() {
    document.removeEventListener("click", this.handleClickOutside)
    clearTimeout(this.timeout)
  }

  search() {
    clearTimeout(this.timeout)
    const query = this.inputTarget.value.trim()

    if (query.length === 0) {
      this.hideDropdown()
      return
    }

    this.timeout = setTimeout(() => {
      this.frameTarget.src = `${this.urlValue}?q=${encodeURIComponent(query)}`
    }, 300)
  }

  keydown(event) {
    if (event.key === "Escape") {
      this.hideDropdown()
      this.inputTarget.blur()
    } else if (event.key === "Enter") {
      event.preventDefault()
      const query = this.inputTarget.value.trim()
      if (query.length > 0) {
        window.Turbo.visit(`/search?q=${encodeURIComponent(query)}`)
      }
    }
  }

  showDropdown() {
    this.dropdownTarget.classList.remove("hidden")
  }

  hideDropdown() {
    this.dropdownTarget.classList.add("hidden")
  }

  handleClickOutside(event) {
    if (!this.element.contains(event.target)) {
      this.hideDropdown()
    }
  }
}
