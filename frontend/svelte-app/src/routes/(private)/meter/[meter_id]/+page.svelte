<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { getContext, onMount } from 'svelte';
  import { goto } from '$app/navigation';

  import type { Chart, ChartEvent } from 'chart.js';
  import type { FluidMeterAlerts } from '@api/FluidMeter';
  import type { Message } from '@api/Message';
  import type { Series } from '@api/Common';

  import {
    FluidMeterStatus,
    activateFluidMeter,
    deactivateFluidMeter,
    deleteFluidMeter,
    getMeasurementsBrowser
  } from '@api/FluidMeter';
  import { SeriesGranularity } from '@api/Common';
  import { MessageType } from '@api/Message';
  import MdModal from '@components/MdModal.svelte';

  const globalMessages: SvelteMap<string, Message> = getContext('globalMessages');

  type Props = {
    data: {
      series: Series;
      alerts: FluidMeterAlerts;
    };
  };

  type chart_point = {
    date: Date;
    label: string;
    litters: number;
  };

  let props: Props = $props();
  let alerts = $state(props.data.alerts);
  let selectedDay: Date | null = $state(null);
  let showDeleteModal = $state(false);
  let graphData: chart_point[] = $state([]);

  // The server returns the data from newest to oldest excluding time frames that
  // have no data.
  // We want to show from oldest to newest and with no holes.
  // Even when server returns data in UTC, we'll treat it as the local time zone
  function buildChartData(start_date: Date, in_data: Series) {
    graphData = [];
    let items = in_data.items;
    if (!items.length) {
      return graphData;
    }

    if (in_data.granularity == SeriesGranularity.Day) {
      let index = items.length - 1;
      for (let i = 0; i < 31; i++) {
        let litters = 0;

        if (index >= 0) {
          let current = items[index];
          let currentDate = new Date(current.period_start);

          if (
            currentDate.getDate() == start_date.getDate() &&
            currentDate.getMonth() == start_date.getMonth()
          ) {
            litters = parseFloat(current.value);
            index--;
          }
        }

        let data = {
          date: new Date(start_date),
          label: `${start_date.getDate()}/${start_date.getMonth() + 1}`,
          litters: litters
        };
        graphData.push(data);

        start_date = new Date(start_date.setDate(start_date.getDate() + 1));
      }
    } else if (in_data.granularity == SeriesGranularity.Hour) {
      let index = items.length - 1;
      for (let i = 0; i < 24; i++) {
        let litters = 0;

        if (index >= 0) {
          let current = items[index];
          let currentHour = new Date(current.period_start).getHours();

          if (currentHour == i) {
            litters = parseFloat(current.value);
            index--;
          }
        }

        let data = {
          date: new Date(start_date.setHours(i)),
          label: '' + i,
          litters: litters
        };

        graphData.push(data);
      }
    }
  }

  let chart: Chart<'line', number[], string>;
  async function createChart(data: chart_point[]) {
    const chartModule = await import('chart.js/auto');
    const chartjs = chartModule.Chart;
    const canvas = document.getElementById('usage') as HTMLCanvasElement;
    const ctx = canvas.getContext('2d');
    if (chart) {
      chart.destroy();
    }
    if (!ctx) {
      return;
    }
    chart = new chartjs(ctx, {
      type: 'line',
      data: {
        labels: data.map((v: chart_point) => v.label),
        datasets: [
          {
            label: 'Litters',
            borderColor: 'rgb(12, 196, 247)',
            backgroundColor: 'rgb(12, 196, 247)',
            data: data.map((v) => v.litters)
          }
        ]
      },
      options: {
        onClick: async (e: ChartEvent) => {
          if (selectedDay) {
            return;
          }

          const nativeEvent = e.native as unknown as Event;

          const points = chart.getElementsAtEventForMode(
            nativeEvent,
            'nearest',
            { intersect: true },
            true
          );
          if (points.length > 0) {
            const index = points[0].index;
            let date = graphData[index].date;
            selectedDay = date;
            let newData = await getMeasurementsBrowser(
              alerts.meter.id,
              SeriesGranularity.Hour,
              date
            );
            if ('code' in newData) {
              let message: Message = {
                type: MessageType.Error,
                text: 'Sorry. We failed to get data.'
              };
              globalMessages.set('retrieve-data-error', message);
            } else {
              buildChartData(date, newData);
              createChart(graphData);
            }
          }
        }
      }
    });
  }

  onMount(async () => {
    let now = new Date();
    let oneMonthAgo = new Date(now.setDate(now.getDate() - 30));
    buildChartData(oneMonthAgo, props.data.series);
    createChart(graphData);
  });

  async function toggleStatus() {
    let r = null;
    let newStatus = FluidMeterStatus.Active;
    if (alerts.meter.status == FluidMeterStatus.Active) {
      r = await deactivateFluidMeter(alerts.meter.id);
      newStatus = FluidMeterStatus.Inactive;
    } else {
      r = await activateFluidMeter(alerts.meter.id);
    }

    if (r == 200) {
      alerts.meter.status = newStatus;
    } else {
      let message: Message = {
        type: MessageType.Error,
        text: 'Sorry. We failed to change the status.'
      };
      globalMessages.set('status-change-error', message);
    }
  }

  function confirmDelete() {
    showDeleteModal = true;
  }

  function cancelDelete() {
    showDeleteModal = false;
  }

  async function showMonthUsage() {
    selectedDay = null;
    let now = new Date();
    let oneMonthAgo = new Date(now.setDate(now.getDate() - 30));
    let newData = await getMeasurementsBrowser(alerts.meter.id, SeriesGranularity.Day, null);
    if ('code' in newData) {
      let message: Message = {
        type: MessageType.Error,
        text: 'Sorry. We failed to get data.'
      };
      globalMessages.set('retrieve-data-error', message);
    } else {
      buildChartData(oneMonthAgo, newData);
      createChart(graphData);
    }
  }

  async function doDelete() {
    if ((await deleteFluidMeter(alerts.meter.id)) == 200) {
      let message: Message = {
        type: MessageType.Success,
        text: 'Meter deleted'
      };
      globalMessages.set('delete-meter-success', message);
      goto('/dashboard');
    } else {
      let message: Message = {
        type: MessageType.Error,
        text: 'Sorry. We failed to delete the meter.'
      };
      globalMessages.set('delete-meter-error', message);
    }
  }
</script>

<div class="container">
  <div class={showDeleteModal ? '' : 'hidden'}>
    <MdModal closeCallback={cancelDelete}>
      <h2>Are you sure you wan't to delete this meter?</h2>
      <div class="title">
        <button class="button" onclick={() => doDelete()}>Delete</button>
        <button class="button" onclick={() => cancelDelete()}>Don't delete</button>
      </div>
    </MdModal>
  </div>
  <div class="title">
    <h1>{alerts.meter.name} ({alerts.meter.id})</h1>
    <button class="button2" onclick={() => confirmDelete()}>Delete</button>
  </div>
  {#if alerts.alerts.length}
    <div class="alerts">
      <p><strong>Alerts</strong>:</p>
      {#each alerts.alerts as a}
        <span class="alert">{a.alert_type}</span>
      {/each}
    </div>
  {/if}
  <div class="st-container">
    <p><strong>Status</strong>: {alerts.meter.status}</p>
    <button class="button" onclick={() => toggleStatus()}>
      {alerts.meter.status == FluidMeterStatus.Active ? 'Deactivate' : 'Activate'}
    </button>
  </div>
  {#if selectedDay}
    <p>{'Usage on ' + new Date(selectedDay).toISOString().split('T')[0]}</p>
    <button class="button" onclick={() => showMonthUsage()}>Show month usage</button>
  {:else}
    <p>{'Usage in the last 30 days'}</p>
  {/if}
  <div style="width: 800px;"><canvas id="usage"></canvas></div>
  <p>
    *Times shown in UTC<br />
    *Click on a data point to see more granular data for that day
  </p>
</div>

<style>
  .container {
    margin: 0 auto;
    margin-top: 1rem;
    width: 80%;
  }

  .alerts,
  .st-container,
  .title {
    display: flex;
  }

  .title button {
    margin: 0 1rem;
    height: 2rem;
    align-self: center;
  }

  .button {
    margin-left: 1rem;
  }

  .st-container p {
    margin: 0;
    line-height: 2rem;
  }

  .alerts {
    margin-bottom: 1rem;
  }

  .alerts p {
    line-height: 2rem;
    margin: 0;
    padding: 0;
  }

  .alert {
    margin: 0 0.2rem;
    padding: 0.5rem;
    border: none;
    border-radius: 100px;
    background: #e3342f;
    color: #fff;
    font-weight: bold;
    font-size: 0.8em;
  }
</style>
