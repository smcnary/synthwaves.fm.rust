module Orderable
  extend ActiveSupport::Concern

  private

  def sort_column(klass, default: "created_at")
    params[:sort].presence_in(klass.sortable_columns) || default
  end

  def sort_direction(default: "asc")
    params[:direction].presence_in(%w[asc desc]) || default
  end
end
