class PlayerComponent < ViewComponent::Base
  def render?
    helpers.authenticated?
  end
end
