package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.communication.Renamed
import net.mullvad.mullvadvpn.compose.component.CustomListNameTextField
import net.mullvad.mullvadvpn.compose.state.EditCustomListNameUiState
import net.mullvad.mullvadvpn.compose.test.EDIT_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GetCustomListError
import net.mullvad.mullvadvpn.lib.model.NameAlreadyExists
import net.mullvad.mullvadvpn.lib.model.UnknownCustomListError
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.usecase.customlists.RenameError
import net.mullvad.mullvadvpn.viewmodel.EditCustomListNameDialogSideEffect
import net.mullvad.mullvadvpn.viewmodel.EditCustomListNameDialogViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewEditCustomListNameDialog() {
    AppTheme { EditCustomListNameDialog(EditCustomListNameUiState()) }
}

data class EditCustomListNameNavArgs(
    val customListId: CustomListId,
    val initialName: CustomListName
)

@Composable
@Destination<RootGraph>(
    style = DestinationStyle.Dialog::class,
    navArgs = EditCustomListNameNavArgs::class
)
fun EditCustomListName(
    backNavigator: ResultBackNavigator<Renamed>,
) {
    val vm: EditCustomListNameDialogViewModel = koinViewModel()
    LaunchedEffectCollect(vm.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            is EditCustomListNameDialogSideEffect.ReturnWithResult -> {
                backNavigator.navigateBack(result = sideEffect.result)
            }
        }
    }

    val state by vm.uiState.collectAsStateWithLifecycle()
    EditCustomListNameDialog(
        state = state,
        updateName = vm::updateCustomListName,
        onInputChanged = vm::onNameChanged,
        onDismiss = dropUnlessResumed { backNavigator.navigateBack() }
    )
}

@Composable
fun EditCustomListNameDialog(
    state: EditCustomListNameUiState,
    updateName: (String) -> Unit = {},
    onInputChanged: (String) -> Unit = {},
    onDismiss: () -> Unit = {}
) {
    AlertDialog(
        title = {
            Text(
                text = stringResource(id = R.string.update_list_name),
            )
        },
        text = {
            CustomListNameTextField(
                name = state.name,
                isValidName = state.isValidName,
                error = state.error?.errorString(),
                onSubmit = updateName,
                onValueChanged = onInputChanged,
                modifier = Modifier.testTag(EDIT_CUSTOM_LIST_DIALOG_INPUT_TEST_TAG)
            )
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        onDismissRequest = onDismiss,
        confirmButton = {
            PrimaryButton(
                text = stringResource(id = R.string.save),
                onClick = { updateName(state.name) },
                isEnabled = state.isValidName
            )
        },
        dismissButton = {
            PrimaryButton(text = stringResource(id = R.string.cancel), onClick = onDismiss)
        }
    )
}

@Composable
private fun RenameError.errorString() =
    stringResource(
        when (error) {
            is NameAlreadyExists -> R.string.custom_list_error_list_exists
            is GetCustomListError,
            is UnknownCustomListError -> R.string.error_occurred
        }
    )
