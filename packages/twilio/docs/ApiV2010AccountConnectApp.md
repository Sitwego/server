# ApiV2010AccountConnectApp

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the ConnectApp resource. | [optional]
**authorize_redirect_url** | Option<**String**> | The URL we redirect the user to after we authenticate the user and obtain authorization to access the Connect App. | [optional]
**company_name** | Option<**String**> | The company name set for the Connect App. | [optional]
**deauthorize_callback_method** | Option<**DeauthorizeCallbackMethod**> | The HTTP method we use to call `deauthorize_callback_url`. (enum: GET, POST) | [optional]
**deauthorize_callback_url** | Option<**String**> | The URL we call using the `deauthorize_callback_method` to de-authorize the Connect App. | [optional]
**description** | Option<**String**> | The description of the Connect App. | [optional]
**friendly_name** | Option<**String**> | The string that you assigned to describe the resource. | [optional]
**homepage_url** | Option<**String**> | The public URL where users can obtain more information about this Connect App. | [optional]
**permissions** | Option<[**Vec<models::ConnectAppEnumPermission>**](ConnectAppEnumPermission.md)> | The set of permissions that your ConnectApp requests. | [optional]
**sid** | Option<**String**> | The unique string that that we created to identify the ConnectApp resource. | [optional]
**uri** | Option<**String**> | The URI of the resource, relative to `https://api.twilio.com`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


