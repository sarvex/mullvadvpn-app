import { useCallback, useEffect, useState } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { IDevice } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { transitions, useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useBoolean } from '../lib/utilityHooks';
import { useSelector } from '../redux/store';
import * as AppButton from './AppButton';
import * as Cell from './cell';
import { bigText } from './common-styles';
import CustomScrollbars from './CustomScrollbars';
import { Brand, HeaderBarSettingsButton } from './HeaderBar';
import ImageView from './ImageView';
import { Header, Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType, ModalContainer, ModalMessage } from './Modal';

const StyledCustomScrollbars = styled(CustomScrollbars)({
  flex: 1,
});

const StyledContainer = styled(SettingsContainer)({
  paddingTop: '14px',
  minHeight: '100%',
});

const StyledBody = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  paddingBottom: 'auto',
});

const StyledFooter = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 0,
  padding: '18px 22px 22px',
});

const StyledStatusIcon = styled.div({
  alignSelf: 'center',
  width: '60px',
  height: '60px',
  marginBottom: '18px',
});

const StyledTitle = styled.span(bigText, {
  lineHeight: '38px',
  margin: '0 22px 8px',
});

const StyledLabel = styled.span({
  fontFamily: 'Open Sans',
  fontSize: '13px',
  fontWeight: 600,
  lineHeight: '20px',
  color: colors.white,
  margin: '0 22px 18px',
});

const StyledDeviceList = styled(Cell.CellButtonGroup)({
  marginBottom: 0,
  flex: '0 0',
});

const StyledSpacer = styled.div({
  flex: '1',
});

const StyledDeviceName = styled(Cell.Label)({
  ':first-letter': {
    textTransform: 'uppercase',
  },
});

export default function TooManyDevices() {
  const history = useHistory();
  const { listDevices, removeDevice, login, cancelLogin } = useAppContext();
  const accountToken = useSelector((state) => state.account.accountToken)!;
  const [devices, setDevices] = useState<Array<IDevice>>();

  const fetchDevices = useCallback(async () => {
    setDevices(await listDevices(accountToken));
  }, [listDevices, accountToken]);

  const onRemoveDevice = useCallback(
    async (deviceId: string) => {
      await removeDevice({ accountToken, deviceId });
      await fetchDevices();
    },
    [removeDevice, accountToken],
  );

  const continueLogin = useCallback(() => login(accountToken), [login, accountToken]);
  const cancel = useCallback(() => {
    cancelLogin();
    history.reset(RoutePath.login, transitions.pop);
  }, [history.reset, cancelLogin]);

  useEffect(() => void fetchDevices(), []);

  const iconSource = getIconSource(devices);
  const title = getTitle(devices);
  const subtitle = getSubtitle(devices);

  return (
    <ModalContainer>
      <Layout>
        <Header>
          <Brand />
          <HeaderBarSettingsButton />
        </Header>
        <StyledCustomScrollbars fillContainer>
          <StyledContainer>
            <StyledBody>
              <StyledStatusIcon>
                <ImageView key={iconSource} source={iconSource} height={60} width={60} />
              </StyledStatusIcon>
              {devices !== undefined && (
                <>
                  <StyledTitle>{title}</StyledTitle>
                  <StyledLabel>{subtitle}</StyledLabel>
                  <DeviceList devices={devices} onRemoveDevice={onRemoveDevice} />
                </>
              )}
            </StyledBody>

            {devices !== undefined && (
              <StyledFooter>
                <AppButton.ButtonGroup>
                  <AppButton.GreenButton onClick={continueLogin} disabled={devices.length === 5}>
                    {messages.pgettext('too-many-devices-view', 'Continue with login')}
                  </AppButton.GreenButton>
                  <AppButton.BlueButton onClick={cancel}>
                    {messages.gettext('Back')}
                  </AppButton.BlueButton>
                </AppButton.ButtonGroup>
              </StyledFooter>
            )}
          </StyledContainer>
        </StyledCustomScrollbars>
      </Layout>
    </ModalContainer>
  );
}

interface IDeviceListProps {
  devices: Array<IDevice>;
  onRemoveDevice: (deviceId: string) => Promise<void>;
}

function DeviceList(props: IDeviceListProps) {
  return (
    <>
      <StyledSpacer>
        <StyledDeviceList>
          {props.devices.map((device) => (
            <Device key={device.id} device={device} onRemove={props.onRemoveDevice} />
          ))}
        </StyledDeviceList>
      </StyledSpacer>
    </>
  );
}

interface IDeviceProps {
  device: IDevice;
  onRemove: (deviceId: string) => Promise<void>;
}

function Device(props: IDeviceProps) {
  const [confirmationVisible, showConfirmation, hideConfirmation] = useBoolean(false);

  const onRemove = useCallback(async () => {
    await props.onRemove(props.device.id);
    hideConfirmation();
  }, [props.onRemove, props.device.id, hideConfirmation]);

  return (
    <>
      <Cell.CellButton>
        <StyledDeviceName>{props.device.name}</StyledDeviceName>
        <ImageView
          source="icon-close"
          onClick={showConfirmation}
          tintColor={colors.white40}
          tintHoverColor={colors.white60}
        />
      </Cell.CellButton>
      {confirmationVisible && (
        <ModalAlert
          type={ModalAlertType.warning}
          iconColor={colors.red}
          buttons={[
            <AppButton.RedButton key="remove" onClick={onRemove}>
              {messages.pgettext('too-many-devices-view', 'Yes, log out device')}
            </AppButton.RedButton>,
            <AppButton.BlueButton key="back" onClick={hideConfirmation}>
              {messages.gettext('Back')}
            </AppButton.BlueButton>,
          ]}
          close={hideConfirmation}>
          <ModalMessage>
            {sprintf(
              messages.pgettext(
                'too-many-devices-view',
                'Are you sure you want to log out of %(deviceName)s?',
              ),
              { deviceName: props.device.name },
            )}
          </ModalMessage>
          <ModalMessage>
            {messages.pgettext(
              'too-many-devices-view',
              'This will delete all forwarded ports. Local settings will be saved.',
            )}
          </ModalMessage>
        </ModalAlert>
      )}
    </>
  );
}

function getIconSource(devices?: Array<IDevice>): string {
  if (devices) {
    if (devices.length === 5) {
      return 'icon-fail';
    } else {
      return 'icon-success';
    }
  } else {
    return 'icon-spinner';
  }
}

function getTitle(devices?: Array<IDevice>): string | undefined {
  if (devices) {
    if (devices.length === 5) {
      return messages.pgettext('too-many-devices-view', 'Too many devices');
    } else {
      return messages.pgettext('too-many-devices-view', 'Super!');
    }
  } else {
    return undefined;
  }
}

function getSubtitle(devices?: Array<IDevice>): string | undefined {
  if (devices) {
    if (devices.length === 5) {
      return messages.pgettext(
        'too-many-devices-view',
        'You have too many active devices. Please log out of at least one by removing it from the list below. You can find the corresponding nickname under the device’s Account settings.',
      );
    } else {
      return messages.pgettext(
        'too-many-devices-view',
        'You can now continue logging in on this device.',
      );
    }
  } else {
    return undefined;
  }
}
