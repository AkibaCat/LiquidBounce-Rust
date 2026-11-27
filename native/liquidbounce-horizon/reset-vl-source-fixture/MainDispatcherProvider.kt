@file:OptIn(kotlinx.coroutines.InternalCoroutinesApi::class)
package fixture
import kotlinx.coroutines.MainCoroutineDispatcher
import kotlinx.coroutines.internal.MainDispatcherFactory
import kotlin.coroutines.CoroutineContext
class ImmediateMain:MainCoroutineDispatcher(){
    override val immediate:MainCoroutineDispatcher get()=this
    override fun isDispatchNeeded(context:CoroutineContext)=false
    override fun dispatch(context:CoroutineContext,block:Runnable){block.run()}
}
class ImmediateMainFactory:MainDispatcherFactory {
    override val loadPriority:Int get()=Int.MAX_VALUE
    override fun createDispatcher(allFactories:List<MainDispatcherFactory>):MainCoroutineDispatcher=ImmediateMain()
    override fun hintOnError():String="oracle immediate render dispatcher"
}
