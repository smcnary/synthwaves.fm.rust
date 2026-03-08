module SortHelper
  def sort_link(relation, column, title, options = {})
    matching_column = column.to_s == params[:sort]
    direction = if matching_column
      (params[:direction] == "asc") ? "desc" : "asc"
    else
      "asc"
    end

    css_classes = ["sortable-link"]
    css_classes << "active" if matching_column
    css_classes += Array(options[:class])

    link_to request.params.merge(sort: column, direction: direction),
      options.merge(class: css_classes.join(" ")) do
      concat title
      concat sort_icon(direction) if matching_column
    end
  end

  def sort_icon(direction)
    if direction == "asc"
      <<~SVG.html_safe
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="currentColor" class="w-4 h-4 inline-block ml-1" aria-label="Sorted ascending">
          <path fill-rule="evenodd" d="M11.78 9.78a.75.75 0 0 1-1.06 0L8 7.06 5.28 9.78a.75.75 0 0 1-1.06-1.06l3.25-3.25a.75.75 0 0 1 1.06 0l3.25 3.25a.75.75 0 0 1 0 1.06Z" clip-rule="evenodd" />
        </svg>
      SVG
    else
      <<~SVG.html_safe
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="currentColor" class="w-4 h-4 inline-block ml-1" aria-label="Sorted descending">
          <path fill-rule="evenodd" d="M4.22 6.22a.75.75 0 0 1 1.06 0L8 8.94l2.72-2.72a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 0-1.06Z" clip-rule="evenodd" />
        </svg>
      SVG
    end
  end
end
