idf_component_register(SRCS "src/main.cpp"
                            "src/backend-service.cpp"
                       INCLUDE_DIRS "include"
                       PRIV_REQUIRES "button"
                                     "esp-idf-wifi-manager"
                                     "esp_driver_gpio"
                                     "esp_http_client"
                                     "esp_wifi"
                                     "fluid-meter"
                                     "mbedtls")

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
