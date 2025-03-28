idf_component_register(SRCS "src/fluid-meter.cpp"
                       INCLUDE_DIRS "include"
                       PRIV_REQUIRES "esp_driver_gpio")


set(MY_FLAGS
    "-Wall"
    "-Wextra"
    "-Werror"
    "-Werror=unused-function"
    "-Werror=unused-variable"
    "-Werror=unused-but-set-variable"
    "-Werror=deprecated-declarations"
)

target_compile_options(${COMPONENT_LIB} PRIVATE ${MY_FLAGS})
