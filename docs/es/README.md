# User documentation

## Indicadores LED

- **Amarillo** - Dispositivo iniciando
- **Verde, amarillo y rojo** - Dispositivo en modo fábrica. Listo para ser configurado (Ver instrucciones abajo para configurar)
- **Verde** - Dispositivo configurado y funcionando correctamente
- **Verde y amarillo** - Dispositivo enviando información a servidores
- **Rojo** - La última petición a los servidores fallo. Esto puede suceder porque el dispositivo está mal configurado o el punto de acceso no está disponible
- **Ninguno** - Esto nunca debería suceder. Significa que algo está funcionando mal

## Configuración

Cuando el dispositivo está en modo fabrica (Indicadores LED verde, amarillo y rojo están encendidos), el dispositivo crea un punto de acceso llamado `my-esp32-ssid`. Conéctate a esa red usando la contraseña `APassword`.

Una vez conectado, visita la dirección `sffm.mekadomus.com`. Verás una página similar a la siguiente:

![Configure device screen](/docs/assets/config-screen.png)

Llena los datos:
- *Red* - El nombre de la red al que se conectará el dispositivo
- *Contraseña* - Contraseña de la red seleccionada
- *ID del medidor* - Identificador para este dispositivo. Debe ser igual a un ID creado desde el panel de control: ([https://console.mekadomus.com](https://console.mekadomus.com))

Después de enviar la información recibirás este mensaje:

![Configuration saved screen](/docs/assets/saved-screen.png)

El dispositivo desactivará el punto de acceso `my-esp32-ssid` y empezará a mandar mediciones a los servidores cada `MS_BETWEEN_POSTS`.

## Modo fábrica

Si necesitar reconfigurar el dispositivo, mantén presionado el botón en el dispositivo por 5 segundos. Los LEDs verde, amarillo y rojo se encenderán de nuevo y el dispositivo estará listo para configurarse.
