// This Quint module was generated. Do not edit.

module trace {
  {% for state in trace %}
    val step{{ loop.index}} = {{ state | quintify }}
  {% endfor %}
}
