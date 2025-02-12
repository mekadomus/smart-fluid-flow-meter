<script lang="ts">
  type Props = {
    data: {
      meter_id: string;
      meter_name: string;
    };
  };
  let props: Props = $props();
  let meter_id = $state(props.data.meter_id);
  let meter_name = $state(props.data.meter_name);
</script>

<div class="success-msg msg">
  Your meter with name <strong>{meter_name}</strong> was created with Id:
  <strong>{meter_id}</strong>.<br /><br />

  Use this Id to configure your device.
</div>

<style>
  .msg {
    margin: 0 auto;
    margin-top: 5rem;
    margin-bottom: 5rem;
    width: 34rem;
  }
</style>
