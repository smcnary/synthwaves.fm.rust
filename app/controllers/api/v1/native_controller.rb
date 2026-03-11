class API::V1::NativeController < ApplicationController
  def credentials
    render json: {
      email: Current.user.email_address,
      subsonic_password: Current.user.subsonic_password,
      theme: Current.user.theme
    }
  end
end
