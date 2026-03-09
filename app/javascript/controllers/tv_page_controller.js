import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  connect() {
    document.dispatchEvent(new CustomEvent("tv:enterTvPage"))
  }

  disconnect() {
    document.dispatchEvent(new CustomEvent("tv:leaveTvPage"))
  }
}
