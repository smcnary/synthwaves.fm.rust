class APIKey < ApplicationRecord
  belongs_to :user

  has_secure_password :secret_key

  validates :name, presence: true
  validates :client_id, presence: true, uniqueness: true

  before_validation :generate_client_id, on: :create

  scope :active, -> { where("expires_at IS NULL OR expires_at > ?", Time.current) }

  def expired?
    expires_at.present? && expires_at < Time.current
  end

  def touch_last_used!(ip_address)
    update_columns(last_used_at: Time.current, last_used_ip: ip_address)
  end

  private

  def generate_client_id
    self.client_id ||= "bc_#{SecureRandom.hex(16)}"
  end
end
